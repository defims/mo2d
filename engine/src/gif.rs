//! GIF89a encoder (bonus format — PNG sequence is the primary export).
//!
//! Fixed 3-3-2 palette (deterministic nearest-color quantization; banding is
//! the acknowledged P2-9 trade-off in docs/mvp.md §6), LZW written in
//! "uncompressed" style (literal codes + periodic clears, min code size 8).

use crate::render::Image;

fn palette_332() -> Vec<[u8; 3]> {
    let mut pal = Vec::with_capacity(256);
    for i in 0..256u32 {
        let r = ((i >> 5) & 7) * 255 / 7;
        let g = ((i >> 2) & 7) * 255 / 7;
        let b = (i & 3) * 255 / 3;
        pal.push([r as u8, g as u8, b as u8]);
    }
    pal
}

fn quantize(px: &[u8]) -> Vec<u8> {
    px.chunks(4)
        .map(|c| {
            // Straight alpha onto white for the palette pick keeps colors stable.
            let a = c[3] as u32;
            let comp = |v: u8| -> u8 { ((v as u32 * a + 255 * (255 - a)) / 255) as u8 };
            let (r, g, b) = (comp(c[0]), comp(c[1]), comp(c[2]));
            (((r as u32) >> 5) << 5 | ((g as u32) >> 5) << 2 | (b as u32) >> 6) as u8
        })
        .collect()
}

pub struct GifFrame<'a> {
    pub img: &'a Image,
    /// Delay in centiseconds.
    pub delay_cs: u16,
}

/// Encode an animated GIF. Returns GIF89a bytes with loop-forever extension.
pub fn encode_gif(frames: &[GifFrame<'_>], width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"GIF89a");
    out.extend_from_slice(&(width as u16).to_le_bytes());
    out.extend_from_slice(&(height as u16).to_le_bytes());
    out.push(0xF7); // global color table, 256 entries
    out.push(0); // background color index
    out.push(0); // pixel aspect ratio

    let pal = palette_332();
    for c in &pal {
        out.extend_from_slice(c);
    }

    // NETSCAPE2.0 loop-forever.
    out.extend_from_slice(&[0x21, 0xFF, 0x0B]);
    out.extend_from_slice(b"NETSCAPE2.0");
    out.extend_from_slice(&[0x03, 0x01, 0x00, 0x00, 0x00]);

    for frame in frames {
        // Graphic control extension.
        out.extend_from_slice(&[0x21, 0xF9, 0x04, 0x04]);
        out.extend_from_slice(&frame.delay_cs.to_le_bytes());
        out.extend_from_slice(&[0x00, 0x00]); // transparent index (none), terminator
        // Image descriptor.
        out.extend_from_slice(&[0x2C]);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(width as u16).to_le_bytes());
        out.extend_from_slice(&(height as u16).to_le_bytes());
        out.push(0x00); // no local table, no interlace

        // LZW image data, min code size 8, "uncompressed" style.
        out.push(8);
        let indices = quantize(&frame.img.px);
        let mut lzw: Vec<u8> = Vec::with_capacity(indices.len() + indices.len() / 250 + 8);
        let clear = 256u32;
        let eoi = 257u32;
        let mut bw = BitWriter::default();
        bw.push(clear, 9);
        let mut since_clear = 0usize;
        for &idx in &indices {
            bw.push(idx as u32, 9);
            since_clear += 1;
            if since_clear >= 253 {
                bw.push(clear, 9);
                since_clear = 0;
            }
        }
        bw.push(eoi, 9);
        lzw.extend_from_slice(&bw.bytes);
        for block in lzw.chunks(255) {
            out.push(block.len() as u8);
            out.extend_from_slice(block);
        }
        out.push(0x00); // block terminator
    }

    out.push(0x3B); // trailer
    out
}

#[derive(Default)]
struct BitWriter {
    bytes: Vec<u8>,
    bit_pos: u32,
}

impl BitWriter {
    /// GIF LZW is little-endian bit packed (LSB first).
    fn push(&mut self, code: u32, bits: u32) {
        for i in 0..bits {
            let bit = (code >> i) & 1;
            if self.bit_pos == 0 {
                self.bytes.push(0);
            }
            if bit == 1 {
                let last = self.bytes.len() - 1;
                self.bytes[last] |= 1 << (self.bit_pos);
            }
            self.bit_pos = (self.bit_pos + 1) % 8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gif_deterministic() {
        let img = Image::new(8, 8);
        let frames = [GifFrame { img: &img, delay_cs: 4 }];
        let a = encode_gif(&frames, 8, 8);
        let b = encode_gif(&frames, 8, 8);
        assert_eq!(a, b);
        assert_eq!(&a[..6], b"GIF89a");
        assert_eq!(*a.last().unwrap(), 0x3B);
    }
}
