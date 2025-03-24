use std::cmp::Ordering;

use anyhow::Result;
use huffman::HuffmanCodeGenerator;
mod huffman;

/// Compresses bits
pub fn deflate(bytes: &[u8]) -> Result<Vec<u8>> {
    // arbitrary -> let's think on this later
    let n_blocks = 2;
    let block_size = bytes.len() / n_blocks;
    let mut blocks = vec![];
    let part1 = &bytes[0..block_size];
    let block = deflate_block_no_compression(part1)?;
    blocks.push(block);
    let part2 = &bytes[block_size..];
    let block = deflate_block_fixed_compression(part2)?;
    // let part = &bytes[0..bytes.len()];
    // let block = deflate_block_fixed_compression(part)?;
    blocks.push(block);

    let last_index = blocks.len() - 1;
    blocks[last_index][0] |= 0x01; //=> mark as final block
    Ok(blocks.into_iter().flatten().collect())
}

// fn deflate_block_with_compression(bytes: &[u8]) -> Result<Vec<u8>> {
//     todo!()
// }

fn deflate_block_fixed_compression(bytes: &[u8]) -> Result<Vec<u8>> {
    let header = 0x02;
    let hfgen = HuffmanCodeGenerator::new_fixed();
    let mut result = vec![];
    let mut last_byte: u8 = header;
    let mut last_b_offset = 3;

    for b in bytes.iter() {
        let i = *b as usize;
        let code = hfgen.code(i);
        let mut code_left_to_apply_len = hfgen.code_len(i);
        let mut code = code.reverse_bits() >> (16 - code_left_to_apply_len);
        while code_left_to_apply_len > 0 {
            let possible_next_offset = last_b_offset + code_left_to_apply_len;
            if possible_next_offset >= 8 {
                let b_to_move = last_b_offset + code_left_to_apply_len - 8;
                let shifted_code = (code << b_to_move) as u8;
                last_byte |= shifted_code;

                let left_code = possible_next_offset - 8;
                code_left_to_apply_len = left_code;
                result.push(last_byte);
                code >>= 8 - b_to_move;
                last_byte = 0;
                last_b_offset = 0;
            } else {
                // code can be fully applied

                let b_to_move = last_b_offset;
                let shifted_code = (code >> b_to_move) as u8;
                last_byte |= shifted_code;
                code_left_to_apply_len = 0;

                last_b_offset = possible_next_offset;
            }
        }
    }
    if last_byte != 0 {
        let end_byte_start = 0 << last_b_offset;
        last_byte |= end_byte_start;
        result.push(last_byte);
        result.push(0);
    } else {
        result.push(0);
    }
    Ok(result)
}

fn deflate_block_no_compression(bytes: &[u8]) -> Result<Vec<u8>> {
    let header: u8 = 0x00; //=> 0 = no compression

    let len = bytes.len() as u16;
    let nlen = !bytes.len() as u16;
    let mut data = vec![
        header,
        bottom_half_16(len),
        top_half_16(len),
        bottom_half_16(nlen),
        top_half_16(nlen),
    ];
    data.extend_from_slice(bytes);
    Ok(data)
}

const fn bottom_half_16(n: u16) -> u8 {
    (n & 0x00ff) as u8
}

const fn top_half_16(n: u16) -> u8 {
    ((n & 0xff00) >> 8) as u8
}

const fn btype_from_byte(b: u8) -> Btype {
    get_btype((b & 0x06) >> 1)
}

pub fn inflate(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![];
    let mut bstart = 0;
    loop {
        let is_final_block = (bytes[bstart] & 0x01) != 0;
        let btype = btype_from_byte(bytes[bstart]);
        let remaining_bytes = &bytes[bstart..];
        let (consumed, block) = match btype {
            Btype::NoCompression => inflate_no_compression(remaining_bytes, bstart + 1)?,
            Btype::CompressedFixed => inflate_block_fixed_compression(remaining_bytes, 3)?,
            _ => todo!(),
        };
        bstart += consumed + 1;
        output.extend_from_slice(&block);
        if is_final_block {
            break;
        }
    }
    Ok(output)
}

struct Inflater<'a> {
    bytes: &'a [u8],
    consumed_bytes: usize,
    current_byte_bits_offset: usize,
    fixed_hf_gen: HuffmanCodeGenerator,
}

impl<'a> Inflater<'a> {
    fn new(bytes: &'a [u8], start_bit_offset: usize) -> Self {
        Self {
            bytes,
            consumed_bytes: 0,
            current_byte_bits_offset: start_bit_offset,
            fixed_hf_gen: HuffmanCodeGenerator::new_fixed(),
        }
    }
    fn consume_code(&mut self) -> u16 {
        let mut current_code = 0;
        let mut code_len = 0;
        loop {
            let current_bit = self.consume_bit();
            current_code <<= 1;
            current_code |= current_bit as u16;
            code_len += 1;
            if let Some(value) = self.fixed_hf_gen.get_code_value(current_code, code_len) {
                return value;
            }
        }
    }
    fn consume_bits(&mut self, n: usize) -> Vec<u8> {
        let mut bits = vec![];
        for _ in 0..n {
            bits.push(self.consume_bit());
        }
        bits
    }
    fn consume_bits_reversed(&mut self, n: usize) -> Vec<u8> {
        let mut bits = vec![0; n];
        for i in (0..n).rev() {
            bits[i] = self.consume_bit();
        }
        bits
    }
    fn consume_bit(&mut self) -> u8 {
        let mut b_idx = self.consumed_bytes;
        let mut offset = self.current_byte_bits_offset;
        let current_byte = self.bytes[b_idx];
        let b_shifted = current_byte >> offset;
        offset += 1;
        if offset >= 8 {
            offset = 0;
            b_idx += 1;
        }
        let current_bit = b_shifted & 1;
        self.current_byte_bits_offset = offset;
        self.consumed_bytes = b_idx;
        current_bit
    }
}

fn inflate_no_compression(bytes: &[u8], bstart: usize) -> Result<(usize, Vec<u8>)> {
    let bytes = &bytes[bstart..];
    let len: usize = (bytes[0] as usize) + bytes[1] as usize * 256;
    let data = &bytes[4..];
    Ok((len + 4, data[..len].to_vec()))
}

/// TODO: also add how many bits consumed in the response
fn inflate_block_fixed_compression(
    bytes: &[u8],
    block_data_start: usize,
) -> Result<(usize, Vec<u8>)> {
    let mut result = vec![];
    let mut inflater = Inflater::new(bytes, block_data_start);
    loop {
        let value = inflater.consume_code();

        match value.cmp(&256) {
            Ordering::Equal => {
                break;
            }
            Ordering::Greater => {
                let l = if (257..265).contains(&value) {
                    (value - 254) as u8
                } else if (265..269).contains(&value) {
                    let l = ((value - 260) * 2 + 1) as u8;
                    let next_bit = inflater.consume_bits_reversed(1);
                    l + bits_to_u8_msb_first(&next_bit)
                } else {
                    todo!()
                };
                // let dist = inflater.consume_code();
                let dist = inflater.consume_bits(5);
                let dist = bits_to_u8_msb_first(&dist);
                let dist = if (0..4).contains(&dist) {
                    dist + 1
                } else if (4..6).contains(&dist) {
                    let dist = dist + (dist - 3);
                    let b = inflater.consume_bits_reversed(1);

                    dist + bits_to_u8_msb_first(&b)
                } else if (6..8).contains(&dist) {
                    let dist = dist + (dist - 5) * 3;
                    let b = inflater.consume_bits_reversed(2);
                    let dist = dist + bits_to_u8_msb_first(&b);
                    if dist == 11 { 10 } else { dist }
                } else if (8..10).contains(&dist) {
                    let dist = if dist == 8 { 17 } else { 25 };
                    let b = inflater.consume_bits_reversed(3);

                    dist + bits_to_u8_msb_first(&b)
                } else if (10..12).contains(&dist) {
                    let dist = if dist == 10 { 33 } else { 49 };
                    let b = inflater.consume_bits_reversed(4);

                    dist + bits_to_u8_msb_first(&b)
                } else if (12..14).contains(&dist) {
                    let dist = if dist == 12 { 65 } else { 97 };
                    let b = inflater.consume_bits_reversed(5);

                    dist + bits_to_u8_msb_first(&b)
                } else if (14..16).contains(&dist) {
                    let dist = if dist == 14 { 129 } else { 193 };
                    let b = inflater.consume_bits(6);

                    dist + bits_to_u8_msb_first(&b)
                } else {
                    todo!()
                };
                if dist > 0 {
                    let r_curr_idx = result.len() - dist as usize;
                    for i in r_curr_idx..(r_curr_idx + l as usize) {
                        result.push(result[i]);
                    }
                }
            }
            _ => {
                let value = value as u8;
                result.push(value);
            }
        }
    }
    Ok((inflater.consumed_bytes + 1, result))
}

fn bits_to_u8_msb_first(bits: &[u8]) -> u8 {
    let len = bits.len();
    let mut sum = 0;
    for (i, bit) in bits.iter().enumerate().take(len) {
        sum |= bit << (len - i - 1);
    }
    sum
}

#[derive(Debug, PartialEq)]
enum Btype {
    NoCompression,
    CompressedFixed,
    CompressedDynamic,
    Reserved,
}
const fn get_btype(bits: u8) -> Btype {
    match bits {
        0 => Btype::NoCompression,
        1 => Btype::CompressedFixed,
        2 => Btype::CompressedDynamic,
        3 => Btype::Reserved,
        _ => panic!("shouldn't be possible for a btype to be this big"),
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::{
        Btype, btype_from_byte, deflate, deflate_block_fixed_compression,
        deflate_block_no_compression, inflate, inflate_block_fixed_compression,
        inflate_no_compression,
    };

    const SIMPLE_STR: &str = "a very normal string. some might say it's the normalest string you've ever seen
        but ok to be honest, we're not 100% sure bout that. there is still some ongoing research. but we're
        kinda sorta right there. almost there. i can say this is a very normal string confidently.";

    #[test]
    fn no_compression() -> Result<()> {
        let s = SIMPLE_STR;
        let s_bytes = s.as_bytes();
        let d = deflate_block_no_compression(s_bytes)?;
        let bytes_without_header = &d[1..];
        let (consumed, got) = inflate_no_compression(bytes_without_header, 0)?;
        assert_eq!(consumed, bytes_without_header.len());
        let got = std::str::from_utf8(&got)?;
        assert_eq!(s, got);
        Ok(())
    }

    #[test]
    fn fixed_compression() -> Result<()> {
        let s = SIMPLE_STR;
        let s_bytes = s.as_bytes();
        println!("{s_bytes:?}");
        let d = deflate_block_fixed_compression(s_bytes)?;
        for b in d.iter() {
            print!("{b:0>8b} ");
        }
        println!();
        let header = d[0];
        assert_eq!(btype_from_byte(header), Btype::CompressedFixed);
        let (consumed, s2_bytes) = inflate_block_fixed_compression(&d, 3)?;
        assert_eq!(s_bytes, s2_bytes);
        assert_eq!(d.len(), consumed);
        Ok(())
    }

    #[test]
    fn sanity() -> Result<()> {
        let s = SIMPLE_STR;
        let s_bytes = s.as_bytes();
        let d = deflate(s_bytes)?;
        // assert!(d.len() < s.as_bytes().len());
        let i = inflate(&d)?;
        let s2 = std::str::from_utf8(&i)?;
        assert_eq!(s2, s);
        Ok(())
    }
}
