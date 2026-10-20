use std::fs;
use std::path::Path;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut s1 = 1u32;
    let mut s2 = 0u32;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

fn make_chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::new();
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(tag);
    chunk.extend_from_slice(data);
    let mut to_crc = Vec::with_capacity(4 + data.len());
    to_crc.extend_from_slice(tag);
    to_crc.extend_from_slice(data);
    chunk.extend_from_slice(&crc32(&to_crc).to_be_bytes());
    chunk
}

fn generate_png(w: u32, h: u32) -> Vec<u8> {
    let mut raw_data = Vec::with_capacity((h * (1 + w * 4)) as usize);
    for y in 0..h {
        raw_data.push(0);
        for x in 0..w {
            let border = 4;
            if x < border || x >= w - border || y < border || y >= h - border {
                raw_data.extend_from_slice(&[24, 24, 36, 255]);
            } else if (y >= 20 && y <= 28 && x >= 14 && x <= 50)
                || (x >= 42 && x <= 50 && y >= 20 && y <= 44)
                || (y >= 36 && y <= 44 && x >= 24 && x <= 50) {
                raw_data.extend_from_slice(&[68, 221, 102, 255]);
            } else if (x - 20) * (x - 20) + (y - 24) * (y - 24) <= 16 {
                raw_data.extend_from_slice(&[238, 68, 68, 255]);
            } else {
                raw_data.extend_from_slice(&[34, 34, 50, 255]);
            }
        }
    }

    let adler = adler32(&raw_data);
    let mut zlib = Vec::new();
    zlib.push(0x78);
    zlib.push(0x01);
    let len = raw_data.len() as u16;
    zlib.push(0x01);
    zlib.extend_from_slice(&len.to_le_bytes());
    zlib.extend_from_slice(&(!len).to_le_bytes());
    zlib.extend_from_slice(&raw_data);
    zlib.extend_from_slice(&adler.to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);

    let mut png = Vec::new();
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    png.extend_from_slice(&make_chunk(b"IHDR", &ihdr));
    png.extend_from_slice(&make_chunk(b"IDAT", &zlib));
    png.extend_from_slice(&make_chunk(b"IEND", &[]));
    png
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let icon_path = Path::new(&manifest_dir).join("icon.png");
    if !icon_path.exists() {
        let png = generate_png(64, 64);
        let _ = fs::write(icon_path, png);
    }
}
