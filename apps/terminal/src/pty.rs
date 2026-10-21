//! Pseudo-terminal : lance un shell attaché à un pty et relaie sa sortie par un canal.
//!
//! Uniquement libc (posix_openpt) : pas de libutil, donc rien de plus à lier dans le bac à sable.

use std::ffi::{CStr, CString};
use std::io;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub struct Pty {
    master: i32,
    child: Child,
    rx: Receiver<Vec<u8>>,
}

fn winsize(cols: u16, rows: u16) -> libc::winsize {
    libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 }
}

impl Pty {
    pub fn spawn(prog: &str, args: &[&str], envs: &[(&str, &str)], cwd: &str, cols: u16, rows: u16) -> io::Result<Pty> {
        unsafe {
            let m = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC);
            if m < 0 {
                return Err(io::Error::last_os_error());
            }
            // grantpt peut échouer dans un espace de noms utilisateur (gid tty non mappé) : sans gravité
            let _ = libc::grantpt(m);
            if libc::unlockpt(m) != 0 {
                let e = io::Error::last_os_error();
                libc::close(m);
                return Err(e);
            }
            let mut name = [0 as libc::c_char; 128];
            if libc::ptsname_r(m, name.as_mut_ptr(), name.len()) != 0 {
                let e = io::Error::last_os_error();
                libc::close(m);
                return Err(e);
            }
            let path = CString::new(CStr::from_ptr(name.as_ptr()).to_bytes()).unwrap();
            let s = libc::open(path.as_ptr(), libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC);
            if s < 0 {
                let e = io::Error::last_os_error();
                libc::close(m);
                return Err(e);
            }
            let ws = winsize(cols, rows);
            libc::ioctl(m, libc::TIOCSWINSZ, &ws);

            let dup = |fd: i32| OwnedFd::from_raw_fd(libc::dup(fd));
            let mut cmd = Command::new(prog);
            cmd.args(args).current_dir(cwd);
            for (k, v) in envs {
                cmd.env(k, v);
            }
            cmd.stdin(Stdio::from(dup(s))).stdout(Stdio::from(dup(s))).stderr(Stdio::from(dup(s)));
            cmd.pre_exec(|| {
                // nouvelle session + le pty devient le terminal de contrôle (nécessaire au contrôle des jobs)
                libc::setsid();
                libc::ioctl(0, libc::TIOCSCTTY as _, 0);
                Ok(())
            });
            let child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    libc::close(s);
                    libc::close(m);
                    return Err(e);
                }
            };
            libc::close(s);

            // lecteur : le thread bloque sur read() et pousse les octets dans le canal
            let rfd = libc::dup(m);
            let (tx, rx) = channel::<Vec<u8>>();
            std::thread::spawn(move || {
                let mut buf = [0u8; 8192];
                loop {
                    let n = libc::read(rfd, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
                    if n < 0 {
                        if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                            continue;
                        }
                        break;
                    }
                    if n == 0 {
                        break;
                    }
                    if tx.send(buf[..n as usize].to_vec()).is_err() {
                        break;
                    }
                }
                libc::close(rfd);
            });
            Ok(Pty { master: m, child, rx })
        }
    }

    pub fn write(&self, data: &[u8]) {
        let mut off = 0;
        let mut tries = 0;
        while off < data.len() && tries < 200 {
            let n = unsafe { libc::write(self.master, data[off..].as_ptr() as *const libc::c_void, data.len() - off) };
            if n < 0 {
                let k = io::Error::last_os_error().kind();
                if k == io::ErrorKind::Interrupted || k == io::ErrorKind::WouldBlock {
                    tries += 1;
                    continue;
                }
                break;
            }
            off += n as usize;
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        let ws = winsize(cols, rows);
        unsafe {
            libc::ioctl(self.master, libc::TIOCSWINSZ, &ws);
        }
    }

    /// Récupère la sortie disponible. `Err(code)` quand le shell est terminé (code de sortie, 128+signal).
    pub fn poll(&mut self, out: &mut Vec<u8>) -> Result<(), i32> {
        loop {
            match self.rx.try_recv() {
                Ok(v) => out.extend_from_slice(&v),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Err(self.exit_code()),
            }
        }
        if let Ok(Some(st)) = self.child.try_wait() {
            // le shell est parti : on vide ce qui reste puis on signale la fin
            while let Ok(v) = self.rx.try_recv() {
                out.extend_from_slice(&v);
            }
            return Err(Self::code(st));
        }
        Ok(())
    }

    fn code(st: std::process::ExitStatus) -> i32 {
        use std::os::unix::process::ExitStatusExt;
        st.code().unwrap_or_else(|| 128 + st.signal().unwrap_or(0))
    }

    fn exit_code(&mut self) -> i32 {
        match self.child.try_wait() {
            Ok(Some(st)) => Self::code(st),
            _ => {
                let _ = self.child.kill();
                self.child.wait().map(Self::code).unwrap_or(0)
            }
        }
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        // fermer le maître envoie SIGHUP au shell ; on le tue quand même pour ne pas laisser de zombie
        unsafe {
            libc::close(self.master);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}