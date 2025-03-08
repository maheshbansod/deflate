pub struct HuffmanCodeGenerator {
    codes: Vec<u32>,
}
impl HuffmanCodeGenerator {
    pub fn new_fixed() -> Self {
        let mut lits = vec![0; 288];
        let mut code = 0;
        let mut min_code = [0u32; 10];
        for i in 1..=9 {
            code = (code + HuffmanCodeGenerator::fixed_bl_count(i - 1)) << 1;
            min_code[i] = code as u32;
        }
        for i in 0..288 {
            let len = HuffmanCodeGenerator::fixed_bl_len(i) as usize;
            if len != 0 {
                lits[i] = min_code[len];
                min_code[len] += 1;
            }
        }
        Self { codes: lits }
    }

    pub fn code(&self, i: usize) -> u32 {
        self.codes[i]
    }

    pub const fn code_len(&self, i: usize) -> u8 {
        HuffmanCodeGenerator::fixed_bl_len(i)
    }

    const fn fixed_bl_len(i: usize) -> u8 {
        if i <= 143 {
            8
        } else if i >= 144 && i <= 255 {
            9
        } else if i >= 256 && i <= 279 {
            7
        } else if i >= 280 && i <= 287 {
            8
        } else {
            0
        }
    }

    const fn fixed_bl_count(i: usize) -> u8 {
        match i {
            7 => 24,
            8 => 144 + 8,
            9 => 255 - 144 + 1,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[test]
    fn it_should_encode() -> Result<()> {
        Ok(())
    }
}
