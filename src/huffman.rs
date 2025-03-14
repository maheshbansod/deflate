use std::collections::HashMap;

pub struct HuffmanCodeGenerator {
    codes: Vec<u16>,
    /// (code_len, code) -> value
    values: HashMap<(u8, u16), u16>,
}
impl HuffmanCodeGenerator {
    pub fn new_fixed() -> Self {
        let mut lits = vec![0; 288];
        let mut rev_lits = HashMap::new();
        let mut code: u16 = 0;
        let mut min_code = [0u16; 10];
        for i in 1..=9 {
            code = (code + HuffmanCodeGenerator::fixed_bl_count(i - 1)) << 1;
            min_code[i] = code;
        }
        for i in 0..288 {
            let len = HuffmanCodeGenerator::fixed_bl_len(i) as usize;
            if len != 0 {
                lits[i] = min_code[len];
                rev_lits.insert((len as u8, min_code[len]), i as u16);
                min_code[len] += 1;
            }
        }
        Self {
            codes: lits,
            values: rev_lits,
        }
    }

    pub fn code(&self, i: usize) -> u16 {
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

    const fn fixed_bl_count(i: usize) -> u16 {
        match i {
            7 => 24,
            8 => 144 + 8,
            9 => 255 - 144 + 1,
            _ => 0,
        }
    }

    pub fn get_code_value(&self, code: u16, code_len: usize) -> Option<u16> {
        if code_len < 7 {
            return None;
        };
        self.values.get(&(code_len as u8, code)).cloned()
    }
}
