//! Émulateur de terminal (sous-ensemble xterm) : grille de cellules, défilement, historique,
//! écran alternatif, couleurs 16 / 256 / 24 bits, jeu de caractères DEC (traits).
//!
//! `Term::feed(octets)` interprète la sortie du shell ; `Term::replies` reçoit les réponses à renvoyer
//! au shell (rapport de position du curseur, identification du terminal).

use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Col {
    Default,
    Idx(u8),
    Rgb(u8, u8, u8),
}

pub const BOLD: u8 = 1;
pub const DIM: u8 = 2;
pub const UNDER: u8 = 4;
pub const INVERSE: u8 = 8;
pub const STRIKE: u8 = 16;
pub const HIDDEN: u8 = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    /// '\0' = moitié droite d'un caractère large.
    pub ch: char,
    pub fg: Col,
    pub bg: Col,
    pub attr: u8,
}

pub const BLANK: Cell = Cell { ch: ' ', fg: Col::Default, bg: Col::Default, attr: 0 };

#[derive(Clone, Copy, Debug)]
struct Pen {
    fg: Col,
    bg: Col,
    attr: u8,
}

const PEN0: Pen = Pen { fg: Col::Default, bg: Col::Default, attr: 0 };

#[derive(Clone, Copy, PartialEq, Eq)]
enum St {
    Ground,
    Esc,
    /// ESC + intermédiaire (ex. `ESC ( 0`) : un octet final à consommer ; contient l'intermédiaire.
    EscInter(u8),
    Csi,
    Osc,
    /// Chaîne ignorée (DCS, APC, PM, SOS) jusqu'à ST.
    Str,
}

pub struct Term {
    pub cols: usize,
    pub rows: usize,
    pub grid: Vec<Vec<Cell>>,
    other: Vec<Vec<Cell>>,
    pub alt_active: bool,
    pub sb: VecDeque<Vec<Cell>>,
    pub sb_max: usize,
    pub cx: usize,
    pub cy: usize,
    wrap_next: bool,
    pen: Pen,
    saved: (usize, usize, Pen, bool),
    top: usize,
    bot: usize,
    pub cursor_visible: bool,
    pub app_cursor: bool,
    pub bracketed_paste: bool,
    autowrap: bool,
    origin: bool,
    insert: bool,
    tabs: Vec<bool>,
    graphics: [bool; 2],
    gl: usize,
    pub title: String,
    pub replies: Vec<u8>,
    st: St,
    buf: Vec<u8>,
    utf: [u8; 4],
    utf_len: usize,
    utf_need: usize,
    esc_in_str: bool,
}

fn blank_line(cols: usize, bg: Col) -> Vec<Cell> {
    vec![Cell { ch: ' ', fg: Col::Default, bg, attr: 0 }; cols]
}

fn is_blank_line(l: &[Cell]) -> bool {
    l.iter().all(|c| c.ch == ' ' && c.bg == Col::Default)
}

/// Largeur d'affichage approximative (0 = combinant, 2 = large).
fn char_width(c: char) -> usize {
    let u = c as u32;
    if u < 0x300 {
        return 1;
    }
    match u {
        0x0300..=0x036F | 0x200B..=0x200F | 0x20D0..=0x20FF | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F => 0,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE6F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD => 2,
        _ => 1,
    }
}

/// Jeu de caractères DEC « dessin de traits » (ESC ( 0).
fn dec_graphics(c: char) -> char {
    match c {
        '`' => '◆',
        'a' => '▒',
        'f' => '°',
        'g' => '±',
        'j' => '┘',
        'k' => '┐',
        'l' => '┌',
        'm' => '└',
        'n' => '┼',
        'o' => '⎺',
        'p' => '⎻',
        'q' => '─',
        'r' => '⎼',
        's' => '⎽',
        't' => '├',
        'u' => '┤',
        'v' => '┴',
        'w' => '┬',
        'x' => '│',
        'y' => '≤',
        'z' => '≥',
        '{' => 'π',
        '|' => '≠',
        '}' => '£',
        '~' => '·',
        _ => c,
    }
}

fn parse_params(b: &[u8]) -> Vec<Option<u32>> {
    let mut out = Vec::new();
    let mut cur: Option<u32> = None;
    for &c in b {
        match c {
            b'0'..=b'9' => cur = Some(cur.unwrap_or(0).saturating_mul(10).saturating_add((c - b'0') as u32).min(1_000_000)),
            b';' | b':' => out.push(cur.take()),
            _ => {}
        }
    }
    if cur.is_some() || !b.is_empty() {
        out.push(cur);
    }
    out
}

impl Term {
    pub fn new(cols: usize, rows: usize) -> Term {
        let (cols, rows) = (cols.max(2), rows.max(2));
        let mut t = Term {
            cols,
            rows,
            grid: vec![blank_line(cols, Col::Default); rows],
            other: vec![blank_line(cols, Col::Default); rows],
            alt_active: false,
            sb: VecDeque::new(),
            sb_max: 3000,
            cx: 0,
            cy: 0,
            wrap_next: false,
            pen: PEN0,
            saved: (0, 0, PEN0, false),
            top: 0,
            bot: rows - 1,
            cursor_visible: true,
            app_cursor: false,
            bracketed_paste: false,
            autowrap: true,
            origin: false,
            insert: false,
            tabs: vec![false; cols],
            graphics: [false; 2],
            gl: 0,
            title: String::new(),
            replies: Vec::new(),
            st: St::Ground,
            buf: Vec::new(),
            utf: [0; 4],
            utf_len: 0,
            utf_need: 0,
            esc_in_str: false,
        };
        t.reset_tabs();
        t
    }

    fn reset_tabs(&mut self) {
        self.tabs = (0..self.cols).map(|i| i % 8 == 0).collect();
    }

    /// Nombre de lignes d'historique visibles (aucune sur l'écran alternatif).
    pub fn visible_sb(&self) -> usize {
        if self.alt_active {
            0
        } else {
            self.sb.len()
        }
    }

    /// Ligne `i` de la vue complète (historique puis écran).
    pub fn vline(&self, i: usize) -> Option<&Vec<Cell>> {
        let n = self.visible_sb();
        if i < n {
            self.sb.get(i)
        } else {
            self.grid.get(i - n)
        }
    }

    /// Séquence d'une flèche (`A` haut, `B` bas, `C` droite, `D` gauche) selon le mode du curseur.
    pub fn arrow(&self, k: char) -> Vec<u8> {
        let mid = if self.app_cursor { 'O' } else { '[' };
        format!("\x1b{mid}{k}").into_bytes()
    }

    // ---------------------------------------------------------------- redimensionnement

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let (cols, rows) = (cols.max(2), rows.max(2));
        if cols == self.cols && rows == self.rows {
            return;
        }
        // écran inactif : simple ajustement
        for l in self.other.iter_mut() {
            l.resize(cols, BLANK);
        }
        self.other.resize(rows, blank_line(cols, Col::Default));
        // écran actif
        for l in self.grid.iter_mut() {
            l.resize(cols, BLANK);
        }
        if rows < self.grid.len() {
            let mut excess = self.grid.len() - rows;
            while excess > 0 && self.grid.len() - 1 > self.cy && is_blank_line(self.grid.last().unwrap()) {
                self.grid.pop();
                excess -= 1;
            }
            while excess > 0 {
                let line = self.grid.remove(0);
                if !self.alt_active {
                    self.sb.push_back(line);
                    if self.sb.len() > self.sb_max {
                        self.sb.pop_front();
                    }
                }
                self.cy = self.cy.saturating_sub(1);
                excess -= 1;
            }
        } else if rows > self.grid.len() {
            let mut need = rows - self.grid.len();
            if !self.alt_active {
                while need > 0 {
                    match self.sb.pop_back() {
                        Some(mut l) => {
                            l.resize(cols, BLANK);
                            self.grid.insert(0, l);
                            self.cy += 1;
                            need -= 1;
                        }
                        None => break,
                    }
                }
            }
            for _ in 0..need {
                self.grid.push(blank_line(cols, Col::Default));
            }
        }
        self.cols = cols;
        self.rows = rows;
        self.top = 0;
        self.bot = rows - 1;
        self.cx = self.cx.min(cols - 1);
        self.cy = self.cy.min(rows - 1);
        self.saved.0 = self.saved.0.min(cols - 1);
        self.saved.1 = self.saved.1.min(rows - 1);
        self.wrap_next = false;
        self.reset_tabs();
    }

    // ---------------------------------------------------------------- opérations de base

    fn blank(&self) -> Cell {
        Cell { ch: ' ', fg: Col::Default, bg: self.pen.bg, attr: 0 }
    }

    fn scroll_up(&mut self, n: usize) {
        let n = n.min(self.bot + 1 - self.top);
        for _ in 0..n {
            let line = self.grid.remove(self.top);
            if !self.alt_active && self.top == 0 {
                self.sb.push_back(line);
                if self.sb.len() > self.sb_max {
                    self.sb.pop_front();
                }
            }
            let bl = blank_line(self.cols, self.pen.bg);
            self.grid.insert(self.bot, bl);
        }
    }

    fn scroll_down(&mut self, n: usize) {
        let n = n.min(self.bot + 1 - self.top);
        for _ in 0..n {
            self.grid.remove(self.bot);
            let bl = blank_line(self.cols, self.pen.bg);
            self.grid.insert(self.top, bl);
        }
    }

    fn linefeed(&mut self) {
        if self.cy == self.bot {
            self.scroll_up(1);
        } else if self.cy + 1 < self.rows {
            self.cy += 1;
        }
    }

    fn reverse_index(&mut self) {
        if self.cy == self.top {
            self.scroll_down(1);
        } else if self.cy > 0 {
            self.cy -= 1;
        }
    }

    fn move_to(&mut self, row: usize, col: usize) {
        let (r0, r1) = if self.origin { (self.top, self.bot) } else { (0, self.rows - 1) };
        self.cy = (r0 + row).min(r1);
        self.cx = col.min(self.cols - 1);
        self.wrap_next = false;
    }

    fn erase(&mut self, y: usize, x0: usize, x1: usize) {
        let b = self.blank();
        let x1 = x1.min(self.cols);
        if let Some(line) = self.grid.get_mut(y) {
            for c in line.iter_mut().take(x1).skip(x0) {
                *c = b;
            }
        }
    }

    fn print(&mut self, ch: char) {
        let ch = if self.graphics[self.gl] { dec_graphics(ch) } else { ch };
        let w = char_width(ch);
        if w == 0 {
            return;
        }
        if self.wrap_next && self.autowrap {
            self.cx = 0;
            self.linefeed();
        }
        self.wrap_next = false;
        if self.cx + w > self.cols {
            if self.autowrap {
                self.cx = 0;
                self.linefeed();
            } else {
                self.cx = self.cols - w;
            }
        }
        let (x, y) = (self.cx, self.cy);
        let cell = Cell { ch, fg: self.pen.fg, bg: self.pen.bg, attr: self.pen.attr };
        let b = self.blank();
        let cols = self.cols;
        let insert = self.insert;
        let line = &mut self.grid[y];
        if insert {
            for _ in 0..w {
                line.insert(x, b);
                line.truncate(cols);
            }
        }
        // on écrase une moitié de caractère large : on efface l'autre moitié
        if line[x].ch == '\0' && x > 0 {
            line[x - 1] = b;
        }
        if x + w < cols && line[x + w].ch == '\0' {
            line[x + w] = b;
        }
        line[x] = cell;
        if w == 2 {
            line[x + 1] = Cell { ch: '\0', ..cell };
        }
        self.cx += w;
        if self.cx >= self.cols {
            self.cx = self.cols - 1;
            self.wrap_next = true;
        }
    }

    fn full_reset(&mut self) {
        let (c, r) = (self.cols, self.rows);
        let title = std::mem::take(&mut self.title);
        let sbm = self.sb_max;
        let sb = std::mem::take(&mut self.sb);
        *self = Term::new(c, r);
        self.title = title;
        self.sb_max = sbm;
        self.sb = sb;
    }

    fn switch_alt(&mut self, on: bool) {
        if on == self.alt_active {
            return;
        }
        std::mem::swap(&mut self.grid, &mut self.other);
        self.alt_active = on;
        if on {
            let b = blank_line(self.cols, Col::Default);
            for l in self.grid.iter_mut() {
                *l = b.clone();
            }
        }
        self.top = 0;
        self.bot = self.rows - 1;
    }

    fn save_cursor(&mut self) {
        self.saved = (self.cx, self.cy, self.pen, self.origin);
    }

    fn restore_cursor(&mut self) {
        let (x, y, pen, origin) = self.saved;
        self.cx = x.min(self.cols - 1);
        self.cy = y.min(self.rows - 1);
        self.pen = pen;
        self.origin = origin;
        self.wrap_next = false;
    }

    // ---------------------------------------------------------------- analyseur

    pub fn feed(&mut self, data: &[u8]) {
        for &b in data {
            self.byte(b);
        }
    }

    fn byte(&mut self, b: u8) {
        match self.st {
            St::Ground => self.ground(b),
            St::Esc => self.esc(b),
            St::EscInter(i) => {
                self.st = St::Ground;
                match i {
                    b'(' => self.graphics[0] = b == b'0',
                    b')' => self.graphics[1] = b == b'0',
                    b'#' if b == b'8' => {
                        // DECALN : remplit l'écran de E
                        for y in 0..self.rows {
                            for x in 0..self.cols {
                                self.grid[y][x] = Cell { ch: 'E', ..BLANK };
                            }
                        }
                    }
                    _ => {}
                }
            }
            St::Csi => {
                match b {
                    0x18 | 0x1A => self.st = St::Ground, // CAN / SUB
                    0x1B => {
                        self.st = St::Esc;
                    }
                    0x20..=0x3F => {
                        if self.buf.len() < 64 {
                            self.buf.push(b);
                        }
                    }
                    0x40..=0x7E => {
                        let buf = std::mem::take(&mut self.buf);
                        self.st = St::Ground;
                        self.csi(&buf, b);
                    }
                    // caractères de contrôle au milieu d'une séquence : exécutés tels quels
                    0x00..=0x1F => self.control(b),
                    _ => {}
                }
            }
            St::Osc => match b {
                0x07 => self.end_osc(),
                0x1B => {
                    self.esc_in_str = true;
                }
                b'\\' if self.esc_in_str => {
                    self.esc_in_str = false;
                    self.end_osc();
                }
                0x18 | 0x1A => self.st = St::Ground,
                _ => {
                    self.esc_in_str = false;
                    if self.buf.len() < 512 {
                        self.buf.push(b);
                    }
                }
            },
            St::Str => match b {
                0x07 => self.st = St::Ground,
                0x1B => self.esc_in_str = true,
                b'\\' if self.esc_in_str => {
                    self.esc_in_str = false;
                    self.st = St::Ground;
                }
                0x18 | 0x1A => self.st = St::Ground,
                _ => self.esc_in_str = false,
            },
        }
    }

    fn end_osc(&mut self) {
        self.st = St::Ground;
        let buf = std::mem::take(&mut self.buf);
        let s = String::from_utf8_lossy(&buf).into_owned();
        if let Some((n, rest)) = s.split_once(';') {
            if n == "0" || n == "2" {
                self.title = rest.chars().take(80).collect();
            }
        }
    }

    fn control(&mut self, b: u8) {
        match b {
            0x08 => {
                // BS
                if self.cx > 0 {
                    self.cx -= 1;
                }
                self.wrap_next = false;
            }
            0x09 => {
                // HT
                let mut x = self.cx + 1;
                while x < self.cols - 1 && !self.tabs[x] {
                    x += 1;
                }
                self.cx = x.min(self.cols - 1);
                self.wrap_next = false;
            }
            0x0A..=0x0C => {
                self.linefeed();
                self.wrap_next = false;
            }
            0x0D => {
                self.cx = 0;
                self.wrap_next = false;
            }
            0x0E => self.gl = 1, // SO
            0x0F => self.gl = 0, // SI
            _ => {}
        }
    }

    fn ground(&mut self, b: u8) {
        if self.utf_need > 0 {
            if (0x80..0xC0).contains(&b) {
                self.utf[self.utf_len] = b;
                self.utf_len += 1;
                if self.utf_len == self.utf_need + 1 {
                    if let Ok(s) = std::str::from_utf8(&self.utf[..self.utf_len]) {
                        if let Some(c) = s.chars().next() {
                            self.print(c);
                        }
                    }
                    self.utf_need = 0;
                    self.utf_len = 0;
                }
                return;
            }
            // séquence UTF-8 interrompue
            self.utf_need = 0;
            self.utf_len = 0;
        }
        match b {
            0x1B => {
                self.st = St::Esc;
            }
            0x00..=0x1F => self.control(b),
            0x20..=0x7E => self.print(b as char),
            0x7F => {}
            0xC2..=0xDF => {
                self.utf[0] = b;
                self.utf_len = 1;
                self.utf_need = 1;
            }
            0xE0..=0xEF => {
                self.utf[0] = b;
                self.utf_len = 1;
                self.utf_need = 2;
            }
            0xF0..=0xF4 => {
                self.utf[0] = b;
                self.utf_len = 1;
                self.utf_need = 3;
            }
            _ => {}
        }
    }

    fn esc(&mut self, b: u8) {
        self.st = St::Ground;
        match b {
            b'[' => {
                self.st = St::Csi;
                self.buf.clear();
            }
            b']' => {
                self.st = St::Osc;
                self.buf.clear();
                self.esc_in_str = false;
            }
            b'P' | b'_' | b'^' | b'X' => {
                self.st = St::Str;
                self.esc_in_str = false;
            }
            b'(' | b')' | b'*' | b'+' | b'#' | b' ' | b'%' => self.st = St::EscInter(b),
            b'7' => self.save_cursor(),
            b'8' => self.restore_cursor(),
            b'D' => self.linefeed(),
            b'E' => {
                self.cx = 0;
                self.linefeed();
            }
            b'H' => {
                if self.cx < self.tabs.len() {
                    self.tabs[self.cx] = true;
                }
            }
            b'M' => self.reverse_index(),
            b'c' => self.full_reset(),
            0x1B => self.st = St::Esc,
            0x00..=0x1F => self.control(b),
            _ => {} // ESC = / ESC > (pavé numérique) et autres : ignorés
        }
    }

    fn csi(&mut self, raw: &[u8], fin: u8) {
        // marqueur privé, paramètres, intermédiaires
        let (marker, rest) = match raw.first() {
            Some(&m) if (b'<'..=b'?').contains(&m) => (m, &raw[1..]),
            _ => (0u8, raw),
        };
        let split = rest.iter().position(|c| (0x20..=0x2F).contains(c)).unwrap_or(rest.len());
        let (pbytes, inter) = rest.split_at(split);
        let ps = parse_params(pbytes);
        let arg = |i: usize, def: usize| -> usize {
            match ps.get(i) {
                Some(Some(v)) => *v as usize,
                _ => def,
            }
        };
        let n1 = arg(0, 1).max(1);
        if !inter.is_empty() && fin != b'p' {
            return; // ex. CSI Ps SP q (forme du curseur) : ignoré
        }
        if marker == b'>' || marker == b'=' {
            if fin == b'c' {
                self.replies.extend_from_slice(b"\x1b[>0;0;0c");
            }
            return;
        }
        if marker == b'?' {
            match fin {
                b'h' | b'l' => {
                    let on = fin == b'h';
                    for p in ps.iter().flatten() {
                        self.dec_mode(*p, on);
                    }
                }
                _ => {}
            }
            return;
        }
        match fin {
            b'A' => self.cy = self.cy.saturating_sub(n1).max(if self.cy >= self.top { self.top } else { 0 }),
            b'B' => self.cy = (self.cy + n1).min(if self.cy <= self.bot { self.bot } else { self.rows - 1 }),
            b'C' | b'a' => {
                self.cx = (self.cx + n1).min(self.cols - 1);
                self.wrap_next = false;
            }
            b'D' => {
                self.cx = self.cx.saturating_sub(n1);
                self.wrap_next = false;
            }
            b'E' => {
                self.cy = (self.cy + n1).min(self.rows - 1);
                self.cx = 0;
                self.wrap_next = false;
            }
            b'F' => {
                self.cy = self.cy.saturating_sub(n1);
                self.cx = 0;
                self.wrap_next = false;
            }
            b'G' | b'`' => {
                self.cx = (n1 - 1).min(self.cols - 1);
                self.wrap_next = false;
            }
            b'H' | b'f' => {
                let r = arg(0, 1).max(1) - 1;
                let c = arg(1, 1).max(1) - 1;
                self.move_to(r, c);
            }
            b'd' => {
                let r = n1 - 1;
                let (r0, r1) = if self.origin { (self.top, self.bot) } else { (0, self.rows - 1) };
                self.cy = (r0 + r).min(r1);
                self.wrap_next = false;
            }
            b'I' => {
                for _ in 0..n1 {
                    self.control(0x09);
                }
            }
            b'J' => {
                let (x, y) = (self.cx, self.cy);
                match arg(0, 0) {
                    0 => {
                        self.erase(y, x, self.cols);
                        for l in y + 1..self.rows {
                            self.erase(l, 0, self.cols);
                        }
                    }
                    1 => {
                        for l in 0..y {
                            self.erase(l, 0, self.cols);
                        }
                        self.erase(y, 0, x + 1);
                    }
                    2 => {
                        for l in 0..self.rows {
                            self.erase(l, 0, self.cols);
                        }
                    }
                    3 => self.sb.clear(),
                    _ => {}
                }
            }
            b'K' => {
                let (x, y) = (self.cx, self.cy);
                match arg(0, 0) {
                    0 => self.erase(y, x, self.cols),
                    1 => self.erase(y, 0, x + 1),
                    2 => self.erase(y, 0, self.cols),
                    _ => {}
                }
            }
            b'L' => {
                if self.cy >= self.top && self.cy <= self.bot {
                    let n = n1.min(self.bot + 1 - self.cy);
                    for _ in 0..n {
                        self.grid.remove(self.bot);
                        let bl = blank_line(self.cols, self.pen.bg);
                        self.grid.insert(self.cy, bl);
                    }
                    self.cx = 0;
                    self.wrap_next = false;
                }
            }
            b'M' => {
                if self.cy >= self.top && self.cy <= self.bot {
                    let n = n1.min(self.bot + 1 - self.cy);
                    for _ in 0..n {
                        self.grid.remove(self.cy);
                        let bl = blank_line(self.cols, self.pen.bg);
                        self.grid.insert(self.bot, bl);
                    }
                    self.cx = 0;
                    self.wrap_next = false;
                }
            }
            b'@' => {
                let b = self.blank();
                let (x, cols) = (self.cx, self.cols);
                let line = &mut self.grid[self.cy];
                for _ in 0..n1.min(cols - x) {
                    line.insert(x, b);
                    line.truncate(cols);
                }
            }
            b'P' => {
                let b = self.blank();
                let (x, cols) = (self.cx, self.cols);
                let line = &mut self.grid[self.cy];
                for _ in 0..n1.min(cols - x) {
                    line.remove(x);
                    line.push(b);
                }
            }
            b'X' => {
                let (x, y) = (self.cx, self.cy);
                self.erase(y, x, x + n1);
            }
            b'S' => self.scroll_up(n1),
            b'T' => self.scroll_down(n1),
            b'm' => self.sgr(&ps),
            b'r' => {
                let t = arg(0, 1).max(1) - 1;
                let bt = arg(1, self.rows).clamp(1, self.rows) - 1;
                if t < bt {
                    self.top = t;
                    self.bot = bt;
                    self.move_to(0, 0);
                }
            }
            b's' => self.save_cursor(),
            b'u' => self.restore_cursor(),
            b'h' | b'l' => {
                let on = fin == b'h';
                for p in ps.iter().flatten() {
                    if *p == 4 {
                        self.insert = on;
                    }
                }
            }
            b'n' => match arg(0, 0) {
                5 => self.replies.extend_from_slice(b"\x1b[0n"),
                6 => {
                    let row = if self.origin { self.cy.saturating_sub(self.top) } else { self.cy } + 1;
                    let r = format!("\x1b[{};{}R", row, self.cx + 1);
                    self.replies.extend_from_slice(r.as_bytes());
                }
                _ => {}
            },
            b'c' => self.replies.extend_from_slice(b"\x1b[?62;c"),
            b'g' => match arg(0, 0) {
                0 => {
                    if self.cx < self.tabs.len() {
                        self.tabs[self.cx] = false;
                    }
                }
                3 => self.tabs.iter_mut().for_each(|t| *t = false),
                _ => {}
            },
            b'Z' => {
                for _ in 0..n1 {
                    let mut x = self.cx.saturating_sub(1);
                    while x > 0 && !self.tabs[x] {
                        x -= 1;
                    }
                    self.cx = x;
                }
                self.wrap_next = false;
            }
            b'p' => {
                // DECSTR (CSI ! p) : réinitialisation douce
                if inter == b"!" {
                    self.pen = PEN0;
                    self.origin = false;
                    self.autowrap = true;
                    self.insert = false;
                    self.cursor_visible = true;
                    self.app_cursor = false;
                    self.top = 0;
                    self.bot = self.rows - 1;
                    self.graphics = [false; 2];
                    self.gl = 0;
                }
            }
            _ => {}
        }
    }

    fn dec_mode(&mut self, p: u32, on: bool) {
        match p {
            1 => self.app_cursor = on,
            6 => {
                self.origin = on;
                self.move_to(0, 0);
            }
            7 => self.autowrap = on,
            25 => self.cursor_visible = on,
            47 | 1047 => self.switch_alt(on),
            1048 => {
                if on {
                    self.save_cursor()
                } else {
                    self.restore_cursor()
                }
            }
            1049 => {
                if on {
                    self.save_cursor();
                    self.switch_alt(true);
                } else {
                    self.switch_alt(false);
                    self.restore_cursor();
                }
            }
            2004 => self.bracketed_paste = on,
            _ => {}
        }
    }

    fn sgr(&mut self, ps: &[Option<u32>]) {
        if ps.is_empty() {
            self.pen = PEN0;
            return;
        }
        let mut i = 0;
        while i < ps.len() {
            let v = ps[i].unwrap_or(0);
            match v {
                0 => self.pen = PEN0,
                1 => self.pen.attr |= BOLD,
                2 => self.pen.attr |= DIM,
                4 | 21 => self.pen.attr |= UNDER,
                7 => self.pen.attr |= INVERSE,
                8 => self.pen.attr |= HIDDEN,
                9 => self.pen.attr |= STRIKE,
                22 => self.pen.attr &= !(BOLD | DIM),
                24 => self.pen.attr &= !UNDER,
                27 => self.pen.attr &= !INVERSE,
                28 => self.pen.attr &= !HIDDEN,
                29 => self.pen.attr &= !STRIKE,
                30..=37 => self.pen.fg = Col::Idx((v - 30) as u8),
                39 => self.pen.fg = Col::Default,
                40..=47 => self.pen.bg = Col::Idx((v - 40) as u8),
                49 => self.pen.bg = Col::Default,
                90..=97 => self.pen.fg = Col::Idx((v - 90 + 8) as u8),
                100..=107 => self.pen.bg = Col::Idx((v - 100 + 8) as u8),
                38 | 48 => {
                    let get = |k: usize| ps.get(k).copied().flatten().unwrap_or(0).min(255) as u8;
                    let col = match ps.get(i + 1).copied().flatten() {
                        Some(5) => {
                            let c = Col::Idx(get(i + 2));
                            i += 2;
                            Some(c)
                        }
                        Some(2) => {
                            let c = Col::Rgb(get(i + 2), get(i + 3), get(i + 4));
                            i += 4;
                            Some(c)
                        }
                        _ => None,
                    };
                    if let Some(c) = col {
                        if v == 38 {
                            self.pen.fg = c;
                        } else {
                            self.pen.bg = c;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}