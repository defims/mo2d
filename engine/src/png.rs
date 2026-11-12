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

// ---------------------------------------------------------------------------
// B5 decode: PNG → RGBA8 Image (decode-on-demand textures)
// ---------------------------------------------------------------------------

/// Decode a PNG (8-bit depth; color types 0 = gray, 2 = RGB, 6 = RGBA;
/// non-interlaced) into an RGBA8 [`Image`]. CRCs and the zlib Adler-32 are
/// verified. Real-world compressed streams are handled via the general
/// DEFLATE decoder (`crate::inflate`); the encoder above stays stored-block.
pub fn decode_png(bytes: &[u8]) -> Result<Image, String> {
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 8 || bytes[..8] != SIG {
        return Err("png: bad signature".into());
    }
    let mut w = 0u32;
    let mut h = 0u32;
    let mut color_type = 0u8;
    let mut idat: Vec<u8> = Vec::new();
    let mut seen_iend = false;
    let mut i = 8usize;
    while i + 8 <= bytes.len() {
        let len = u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
        let kind = &bytes[i + 4..i + 8];
        let data_start = i + 8;
        let data_end = data_start + len;
        if data_end + 4 > bytes.len() {
            return Err("png: chunk overruns file".into());
        }
        let data = &bytes[data_start..data_end];
        let mut crc_input = Vec::with_capacity(4 + data.len());
        crc_input.extend_from_slice(kind);
        crc_input.extend_from_slice(data);
        let crc = u32::from_be_bytes([bytes[data_end], bytes[data_end + 1], bytes[data_end + 2], bytes[data_end + 3]]);
        if crc32(&crc_input) != crc {
            return Err(format!("png: chunk {} CRC mismatch", String::from_utf8_lossy(kind)));
        }
        match kind {
            b"IHDR" => {
                if data.len() != 13 {
                    return Err("png: IHDR must be 13 bytes".into());
                }
                w = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                h = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                let depth = data[8];
                color_type = data[9];
                if w == 0 || h == 0 || w > 16384 || h > 16384 {
                    return Err(format!("png: size {}×{} out of range", w, h));
                }
                if depth != 8 {
                    return Err(format!("png: bit depth {} unsupported (only 8)", depth));
                }
                if !matches!(color_type, 0 | 2 | 6) {
                    return Err(format!("png: color type {} unsupported (0=gray, 2=RGB, 6=RGBA)", color_type));
                }
                if data[12] != 0 {
                    return Err("png: interlaced PNGs unsupported".into());
                }
            }
            b"PLTE" => return Err("png: palette PNGs unsupported (save as RGB/RGBA)".into()),
            b"IDAT" => idat.extend_from_slice(data),
            b"IEND" => {
                seen_iend = true;
                break;
            }
            _ => {} // ancillary chunks ignored
        }
        i = data_end + 4;
    }
    if !seen_iend {
        return Err("png: missing IEND".into());
    }
    if w == 0 || h == 0 {
        return Err("png: missing IHDR".into());
    }
    if idat.is_empty() {
        return Err("png: missing IDAT".into());
    }
    let raw = crate::inflate::zlib_decompress(&idat)?;
    let channels: usize = match color_type {
        0 => 1,
        2 => 3,
        _ => 4,
    };
    let stride = w as usize * channels;
    let want = (stride + 1) * h as usize;
    if raw.len() != want {
        return Err(format!("png: decompressed size {} does not match {}×{} (want {})", raw.len(), w, h, want));
    }
    // Unfilter (RFC 2083 §6: None/Sub/Up/Average/Paeth).
    let bpp = channels;
    let mut lines: Vec<Vec<u8>> = Vec::with_capacity(h as usize);
    for y in 0..h as usize {
        let ft = raw[y * (stride + 1)];
        let src = &raw[y * (stride + 1) + 1..y * (stride + 1) + 1 + stride];
        let prev: &[u8] = if y == 0 { &[] } else { &lines[y - 1] };
        let mut row = vec![0u8; stride];
        for x in 0..stride {
            let a = if x >= bpp { row[x - bpp] } else { 0 };
            let b = if y > 0 { prev[x] } else { 0 };
            let c = if x >= bpp && y > 0 { prev[x - bpp] } else { 0 };
            row[x] = match ft {
                0 => src[x],
                1 => src[x].wrapping_add(a),
                2 => src[x].wrapping_add(b),
                3 => src[x].wrapping_add(((a as u16 + b as u16) / 2) as u8),
                4 => {
                    let p = a as i32 + b as i32 - c as i32;
                    let pa = (p - a as i32).abs();
                    let pb = (p - b as i32).abs();
                    let pc = (p - c as i32).abs();
                    let pred = if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c };
                    src[x].wrapping_add(pred)
                }
                _ => return Err(format!("png: unknown filter type {}", ft)),
            };
        }
        lines.push(row);
    }
    // Expand to RGBA8.
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let o = ((y * w as usize + x) * 4) as usize;
            let s = x * channels; // offset within this row
            match color_type {
                0 => {
                    px[o] = lines[y][s];
                    px[o + 1] = lines[y][s];
                    px[o + 2] = lines[y][s];
                    px[o + 3] = 255;
                }
                2 => {
                    px[o..o + 3].copy_from_slice(&lines[y][s..s + 3]);
                    px[o + 3] = 255;
                }
                _ => px[o..o + 4].copy_from_slice(&lines[y][s..s + 4]),
            }
        }
    }
    Ok(Image { w, h, px })
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

    #[test]
    fn decode_roundtrips_engine_encoder() {
        // Encode (stored blocks) → decode: pixels identical.
        let mut img = Image::new(13, 7);
        for y in 0..7u32 {
            for x in 0..13u32 {
                let o = ((y * 13 + x) * 4) as usize;
                img.px[o] = (x * 19 % 256) as u8;
                img.px[o + 1] = (y * 33 % 256) as u8;
                img.px[o + 2] = ((x + y) * 11 % 256) as u8;
                img.px[o + 3] = 255;
            }
        }
        let back = decode_png(&encode_png(&img)).unwrap();
        assert_eq!(back.w, 13);
        assert_eq!(back.h, 7);
        assert_eq!(back.px, img.px);
    }

    #[test]
    fn decodes_real_compressed_fixture() {
        // demo/assets/decode-fixture.png: 8×6 RGBA zlib-compressed (level 9 —
        // dynamic-Huffman deflate), generated by the fixture script. Pixel
        // law: r = x·32, g = y·40, b = (x+y)·17 (mod 256), a = 255.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("demo/assets/decode-fixture.png");
        let bytes = std::fs::read(&path).expect("fixture present");
        let img = decode_png(&bytes).unwrap();
        assert_eq!((img.w, img.h), (8, 6));
        for y in 0..6u32 {
            for x in 0..8u32 {
                let o = ((y * 8 + x) * 4) as usize;
                assert_eq!(img.px[o], ((x * 32) % 256) as u8, "r at ({}, {})", x, y);
                assert_eq!(img.px[o + 1], ((y * 40) % 256) as u8, "g at ({}, {})", x, y);
                assert_eq!(img.px[o + 2], (((x + y) * 17) % 256) as u8, "b at ({}, {})", x, y);
                assert_eq!(img.px[o + 3], 255);
            }
        }
    }

    #[test]
    fn rejects_garbage_and_missing_chunks() {
        assert!(decode_png(b"not a png").is_err());
        let mut img_bytes = encode_png(&Image::new(2, 2));
        // Corrupt a payload byte inside the IDAT (keep CRC → zlib error path).
        let last = img_bytes.len() - 1;
        img_bytes[20] ^= 0xFF;
        let _ = last;
        assert!(decode_png(&img_bytes).is_err(), "corrupted stream must be rejected");
    }
}
