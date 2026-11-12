//! PNG encoder (8-bit RGBA, filter 0, stored zlib blocks).
//!
//! Stored (uncompressed) deflate keeps the encoder trivially deterministic —
//! PNG bytes are hash-frozen acceptance artifacts (mvp.md A1 3D scene gate).

use crate::render::Image;

pub fn encode_png(img: &Image) -> Vec<u8> {
    let mut out = Vec::with_capacity((img.w * img.h * 4) as usize / 2 + 1024);
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);

    // IHDR
    let mut ihdr: Vec<u8> = Vec::with_capacity(13);
    ihdr.extend_from_slice(&img.w.to_be_bytes());
    ihdr.extend_from_slice(&img.h.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type RGBA
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    chunk(&mut out, b"IHDR", &ihdr);

    // IDAT: zlib stream of scanlines, each prefixed with filter byte 0.
    let mut raw: Vec<u8> = Vec::with_capacity((img.w * img.h * 4 + img.h) as usize);
    for y in 0..img.h {
        raw.push(0u8);
        let start = (y * img.w * 4) as usize;
        raw.extend_from_slice(&img.px[start..start + (img.w * 4) as usize]);
    }
    let zlib = zlib_stored(&raw);
    chunk(&mut out, b"IDAT", &zlib);

    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 65535 * 5 + 16);
    out.push(0x78); // CMF: deflate, 32K window
    out.push(0x01); // FLG: check bits, no dict, fastest
    let mut chunks = data.chunks(65535).peekable();
    if data.is_empty() {
        out.push(1);
        out.extend_from_slice(&[0, 0, 0xFF, 0xFF]);
    }
    while let Some(block) = chunks.next() {
        let last = chunks.peek().is_none();
        out.push(if last { 1 } else { 0 });
        let len = block.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for n in 0..256u32 {
        let mut c = n;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
        }
        table[n as usize] = c;
    }
    table
}

fn crc32(data: &[u8]) -> u32 {
    let table = crc_table();
    let mut c: u32 = 0xFFFFFFFF;
    for &byte in data {
        c = table[((c ^ byte as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sha::sha256;

    #[test]
    fn png_signature_and_determinism() {
        let img = Image::new(4, 4);
        let a = encode_png(&img);
        let b = encode_png(&img);
        assert_eq!(a, b);
        assert_eq!(&a[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        assert_eq!(sha256(&a).len(), 32);
    }
}
