use anyhow::Result;

/// Compresses bits
pub fn deflate(bytes: &[u8]) -> Result<Vec<u8>> {
    // let's think of just one block for now
    // so this block is the final block
    // and let's go with the easiest case - no compression
    let mut block = deflate_block_no_compression(bytes)?;

    block[0] = block[0] | 0x80; //=> mark as final block
    Ok(block)
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

pub fn inflate(bytes: &[u8]) -> Result<Vec<u8>> {
    let is_final_block = (bytes[0] & 0x80) != 0;
    if !is_final_block {
        todo!()
    }
    let btype = (bytes[0] & 0x60) >> 5;
    let btype = get_btype(btype);
    let block = match btype {
        Btype::NoCompression => inflate_no_compression(&bytes[1..])?,
        _ => todo!(),
    };
    Ok(block.to_vec())
}

fn inflate_no_compression(bytes: &[u8]) -> Result<Vec<u8>> {
    let len: usize = bytes[0] as usize * 256 + bytes[1] as usize;
    let data = &bytes[4..];
    Ok(data[..len].to_vec())
}

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
        _ => panic!("shouldn't be a possible for a btype to be this big"),
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::{deflate, deflate_block_no_compression, inflate};

    #[test]
    fn no_compression() -> Result<()> {
        let s = "a very normal string";
        let s_bytes = s.as_bytes();
        let d = deflate_block_no_compression(s_bytes)?;
        let s2_bytes = &d[5..];
        assert_eq!(s_bytes, s2_bytes);
        Ok(())
    }

    #[test]
    fn sanity() -> Result<()> {
        let s = "a very normal string";
        let s_bytes = s.as_bytes();
        let d = deflate(s_bytes)?;
        // assert!(d.len() < s.as_bytes().len());
        let i = inflate(&d)?;
        let s2 = std::str::from_utf8(&i)?;
        assert_eq!(s2, s);
        Ok(())
    }
}
