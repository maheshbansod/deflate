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
    blocks.push(block);

    let last_index = blocks.len() - 1;
    blocks[last_index][0] = blocks[last_index][0] | 0x80; //=> mark as final block
    Ok(blocks.into_iter().flatten().collect())
}

fn deflate_block_fixed_compression(bytes: &[u8]) -> Result<Vec<u8>> {
    let header = 0x20;
    let hfgen = HuffmanCodeGenerator::new_fixed();
    let mut result = vec![header];
    let mut last_b_offset = 0;
    let mut last_byte: u8 = 0;

    for b in bytes {
        let i = *b as usize;
        let code = hfgen.code(i);
        let mut code_left_to_apply_len = hfgen.code_len(i);
        while code_left_to_apply_len > 0 {
            let possible_next_offset = last_b_offset + code_left_to_apply_len;
            if possible_next_offset >= 8 {
                let b_to_move = last_b_offset + code_left_to_apply_len - 8;
                let shifted_code = (code >> b_to_move) as u8;
                last_byte = last_byte | shifted_code;

                let left_code = possible_next_offset - 8;
                code_left_to_apply_len = left_code;
                result.push(last_byte);
                last_byte = 0;
                last_b_offset = 0;
            } else {
                // code can be fully applied

                let b_to_move = 8 - possible_next_offset;
                let shifted_code = (code << b_to_move) as u8;
                last_byte |= shifted_code;
                code_left_to_apply_len = 0;

                last_b_offset = possible_next_offset;
            }
        }
    }
    if last_byte != 0 {
        let end_byte_start = 0 << (7 - last_b_offset);
        last_byte |= end_byte_start;
        result.push(last_byte);
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
        top_half_16(len),
        bottom_half_16(len),
        top_half_16(nlen),
        bottom_half_16(nlen),
    ];
    data.extend_from_slice(&bytes);
    Ok(data)
}

const fn bottom_half_16(n: u16) -> u8 {
    (n & 0x00ff) as u8
}

const fn top_half_16(n: u16) -> u8 {
    ((n & 0xff00) >> 8) as u8
}

const fn btype_from_byte(b: u8) -> Btype {
    get_btype((b & 0x60) >> 5)
}

pub fn inflate(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![];
    let mut bstart = 0;
    loop {
        let is_final_block = (bytes[bstart] & 0x80) != 0;
        let btype = btype_from_byte(bytes[bstart]);
        let remaining_bytes = &bytes[(bstart + 1)..];
        let (consumed, block) = match btype {
            Btype::NoCompression => inflate_no_compression(remaining_bytes)?,
            Btype::CompressedFixed => inflate_block_fixed_compression(remaining_bytes)?,
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

fn inflate_no_compression(bytes: &[u8]) -> Result<(usize, Vec<u8>)> {
    let len: usize = bytes[0] as usize * 256 + bytes[1] as usize;
    let data = &bytes[4..];
    Ok((len + 4, data[..len].to_vec()))
}

fn inflate_block_fixed_compression(bytes: &[u8]) -> Result<(usize, Vec<u8>)> {
    let hfgen = HuffmanCodeGenerator::new_fixed();
    let mut current_byte_index = 0;
    let mut current_code: u16 = 0;
    let mut current_code_len = 0;
    let mut current_byte = bytes[current_byte_index];
    let mut current_byte_consumed_len = 0;
    let mut result = vec![];
    loop {
        let current_bit = (current_byte & 0x80) >> 7;
        current_code = current_code << 1;
        current_code = current_code | (current_bit as u16);
        current_byte <<= 1;
        current_byte_consumed_len += 1;
        current_code_len += 1;
        if let Some(value) = hfgen.get_code_value(current_code, current_code_len) {
            if value == 256 {
                break;
            }
            let value = value as u8;
            result.push(value);
            current_code_len = 0;
            current_code = 0;
        }
        if current_byte_consumed_len == 8 {
            current_byte_consumed_len = 0;
            current_byte_index += 1;
            current_byte = bytes[current_byte_index];
        }
    }
    Ok((current_byte_index + 1, result))
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
    };

    const SIMPLE_STR: &str = "a very normal string. some might say it's the normalest string you've ever seen
        but ok to be honest, we're not 100% sure bout that. there is still some ongoing research. but we're
        kinda sorta right there. almost there. i can say this is a very normal string confidently.";

    #[test]
    fn no_compression() -> Result<()> {
        let s = SIMPLE_STR;
        let s_bytes = s.as_bytes();
        let d = deflate_block_no_compression(s_bytes)?;
        let s2_bytes = &d[5..];
        assert_eq!(s_bytes, s2_bytes);
        Ok(())
    }

    #[test]
    fn fixed_compression() -> Result<()> {
        let s = SIMPLE_STR;
        let s_bytes = s.as_bytes();
        let d = deflate_block_fixed_compression(s_bytes)?;
        let header = d[0];
        assert_eq!(btype_from_byte(header), Btype::CompressedFixed);
        let bytes_without_header = &d[1..];
        let (consumed, s2_bytes) = inflate_block_fixed_compression(&bytes_without_header)?;
        assert_eq!(s_bytes, s2_bytes);
        assert_eq!(bytes_without_header.len(), consumed);
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
