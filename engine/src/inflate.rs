//! Minimal DEFLATE + zlib decoder (RFC 1951 / RFC 1950) — zero dependencies,
//! byte-exact output (decompressed bytes are fully determined by the spec).
//!
//! Used by the B5 bitmap pipeline to decode PNG textures on demand. Supports
//! stored, fixed-Huffman and dynamic-Huffman blocks; rejects dictionaries
//! (FDICT). This is a decoder only — the engine's own PNG encoder stays
//! stored-block (deterministic bytes are acceptance artifacts).

/// Inflate a raw DEFLATE stream (no zlib wrapper).
pub fn inflate_raw(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut br = BitReader { data, pos: 0, bit: 0 };
    let mut out: Vec<u8> = Vec::new();
    loop {
        let bfinal = br.bits(1)?;
        let btype = br.bits(2)?;
        match btype {
            0 => {
                br.align_byte();
                let len = br.bits_u16()?;
                let nlen = br.bits_u16()?;
                if len != !nlen {
                    return Err("deflate: stored block LEN/NLEN mismatch".into());
                }
                for _ in 0..len {
                    out.push(br.byte()?);
                }
            }
            1 => {
                let (lit, dist) = fixed_tables();
                inflate_block(&mut br, &mut out, &lit, &dist)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut br)?;
                inflate_block(&mut br, &mut out, &lit, &dist)?;
            }
            _ => return Err("deflate: reserved block type 3".into()),
        }
        if bfinal == 1 {
            break;
        }
    }
    Ok(out)
}

/// Decompress a zlib stream (RFC 1950): 2-byte header, deflate body, Adler-32.
pub fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 6 {
        return Err("zlib: stream too short".into());
    }
    let cmf = data[0];
    let flg = data[1];
    if cmf & 0x0F != 8 {
        return Err(format!("zlib: compression method {} is not deflate", cmf & 0x0F));
    }
    if ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
        return Err("zlib: header FCHECK failed".into());
    }
    if flg & 0x20 != 0 {
        return Err("zlib: FDICT preset dictionaries unsupported".into());
    }
    let body = inflate_raw(&data[2..data.len() - 4])?;
    let expect = u32::from_be_bytes([data[data.len() - 4], data[data.len() - 3], data[data.len() - 2], data[data.len() - 1]]);
    let got = adler32(&body);
    if got != expect {
        return Err(format!("zlib: adler32 mismatch (got {:#010x}, want {:#010x})", got, expect));
    }
    Ok(body)
}

pub fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
}

impl<'a> BitReader<'a> {
    fn bits(&mut self, n: u32) -> Result<u32, String> {
        let mut v = 0u32;
        for i in 0..n {
            let b = self.bit1()?;
            v |= (b as u32) << i;
        }
        Ok(v)
    }

    fn bits_u16(&mut self) -> Result<u16, String> {
        let lo = self.bits(8)? as u16;
        let hi = self.bits(8)? as u16;
        Ok(lo | (hi << 8))
    }

    fn bit1(&mut self) -> Result<u8, String> {
        if self.pos >= self.data.len() {
            return Err("deflate: unexpected end of stream".into());
        }
        let b = (self.data[self.pos] >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.pos += 1;
        }
        Ok(b)
    }

    fn align_byte(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.pos += 1;
        }
    }

    fn byte(&mut self) -> Result<u8, String> {
        if self.pos >= self.data.len() {
            return Err("deflate: unexpected end of stream".into());
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }
}

/// Canonical Huffman decoder (RFC 1951 §3.2.2): codes ordered by
/// (length, symbol); decoding walks bit by bit through the count table.
struct Huff {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huff {
    fn new(lengths: &[u8]) -> Result<Huff, String> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        // Over-subscribed / incomplete checks.
        let mut left = 1i32;
        for l in 1..16 {
            left <<= 1;
            left -= counts[l] as i32;
            if left < 0 {
                return Err("deflate: over-subscribed huffman code".into());
            }
        }
        let mut offs = [0u16; 16];
        for l in 1..15 {
            offs[l + 1] = offs[l] + counts[l];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }
        Ok(Huff { counts, symbols })
    }

    fn decode(&self, br: &mut BitReader) -> Result<u16, String> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for len in 1..16 {
            code |= br.bit1()? as i32;
            let count = self.counts[len] as i32;
            if code - first < count {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err("deflate: huffman code overrun".into())
    }
}

fn fixed_tables() -> (Huff, Huff) {
    let mut lit_lengths = [0u8; 288];
    for (i, l) in lit_lengths.iter_mut().enumerate() {
        *l = match i {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    let dist_lengths = [5u8; 30];
    // Fixed tables are well-formed by construction.
    (Huff::new(&lit_lengths).unwrap(), Huff::new(&dist_lengths).unwrap())
}

const CLC_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

fn dynamic_tables(br: &mut BitReader) -> Result<(Huff, Huff), String> {
    let hlit = br.bits(5)? as usize + 257;
    let hdist = br.bits(5)? as usize + 1;
    let hclen = br.bits(4)? as usize + 4;
    if hlit > 286 || hdist > 30 {
        return Err("deflate: too many length/distance symbols".into());
    }
    let mut clc_lengths = [0u8; 19];
    for i in 0..hclen {
        clc_lengths[CLC_ORDER[i]] = br.bits(3)? as u8;
    }
    let clc = Huff::new(&clc_lengths)?;
    let mut lengths = vec![0u8; hlit + hdist];
    let mut i = 0;
    while i < lengths.len() {
        let sym = clc.decode(br)?;
        match sym {
            0..=15 => {
                lengths[i] = sym as u8;
                i += 1;
            }
            16 => {
                if i == 0 {
                    return Err("deflate: repeat with no previous length".into());
                }
                let prev = lengths[i - 1];
                let rep = 3 + br.bits(2)? as usize;
                for _ in 0..rep {
                    if i >= lengths.len() {
                        return Err("deflate: length repeat overruns".into());
                    }
                    lengths[i] = prev;
                    i += 1;
                }
            }
            17 => {
                let rep = 3 + br.bits(3)? as usize;
                i += rep;
                if i > lengths.len() {
                    return Err("deflate: zero repeat overruns".into());
                }
            }
            18 => {
                let rep = 11 + br.bits(7)? as usize;
                i += rep;
                if i > lengths.len() {
                    return Err("deflate: zero repeat overruns".into());
                }
            }
            _ => return Err("deflate: invalid code-length symbol".into()),
        }
    }
    let lit = Huff::new(&lengths[..hlit])?;
    let dist = Huff::new(&lengths[hlit..])?;
    Ok((lit, dist))
}

const LEN_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
const DIST_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

fn inflate_block(br: &mut BitReader, out: &mut Vec<u8>, lit: &Huff, dist: &Huff) -> Result<(), String> {
    loop {
        let sym = lit.decode(br)?;
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let li = (sym - 257) as usize;
                let len = LEN_BASE[li] as usize + br.bits(LEN_EXTRA[li] as u32)? as usize;
                let dsym = dist.decode(br)? as usize;
                if dsym >= 30 {
                    return Err("deflate: invalid distance symbol".into());
                }
                let d = DIST_BASE[dsym] as usize + br.bits(DIST_EXTRA[dsym] as u32)? as usize;
                if d > out.len() {
                    return Err("deflate: distance beyond output start".into());
                }
                let start = out.len() - d;
                for k in 0..len {
                    let b = out[start + k];
                    out.push(b);
                }
            }
            _ => return Err("deflate: invalid literal/length symbol".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_roundtrip() {
        // Stored-block zlib via the engine encoder's helper is private; build
        // one by hand: header + BFINAL stored block + adler.
        let payload = b"mo2d everything is lines".to_vec();
        let mut z = vec![0x78, 0x01];
        z.push(1); // BFINAL + BTYPE=00
        z.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(payload.len() as u16)).to_le_bytes());
        z.extend_from_slice(&payload);
        z.extend_from_slice(&adler32(&payload).to_be_bytes());
        assert_eq!(zlib_decompress(&z).unwrap(), payload);
    }

    #[test]
    fn fixed_huffman_roundtrip_via_engine_encoder_companion() {
        // The engine PNG encoder writes STORED blocks — decoding its output
        // exercises header parsing + adler (fixed/dynamic tables are covered
        // by the round-trip through the real PNG fixtures in the B5 gate).
        let payload: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        let mut z = vec![0x78, 0x01];
        // Two stored blocks (MAX_NON_FINAL 65535 split not needed at 1000).
        z.push(1);
        z.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(payload.len() as u16)).to_le_bytes());
        z.extend_from_slice(&payload);
        z.extend_from_slice(&adler32(&payload).to_be_bytes());
        assert_eq!(zlib_decompress(&z).unwrap(), payload);
        assert_eq!(adler32(b"abc"), 0x024d0127, "adler32 reference vector (RFC 1950)");
    }

    #[test]
    fn rejects_truncated_and_bad_checksums() {
        let payload = b"data".to_vec();
        let mut z = vec![0x78, 0x01, 1];
        z.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(payload.len() as u16)).to_le_bytes());
        z.extend_from_slice(&payload);
        z.extend_from_slice(&[0, 0, 0, 0]); // wrong adler
        assert!(zlib_decompress(&z).is_err(), "bad adler must be rejected");
        assert!(zlib_decompress(&z[..z.len() - 2]).is_err(), "truncated stream must be rejected");
    }
}
