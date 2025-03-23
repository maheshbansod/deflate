use anyhow::Result;
use deflate::{deflate, inflate};

fn main() -> Result<()> {
    let f = include_bytes!("../test_data/fixed-comp-deflate.deflate");
    let inflated = inflate(f)?;
    println!("s:{}", String::from_utf8_lossy(&inflated).to_string());
    let correct_text = "Some text. Oh text.. oh text oh text ooh.. yayyyy ayayayya.
    I am a text fan aaaahhh.. I\"m a text fan. I need just text. Gimme the text! Gimme.";
    let deflated = deflate(correct_text.as_bytes())?;

    println!("mydeflated: ");
    // for (i, d) in deflated.iter().enumerate() {
    //     print!("{d:b} ");
    //     if f[i] != *d {
    //         println!();
    //         println!("{i}th byte unequal: md:{d:b},actual:{:b}", f[i]);
    //     }
    // }
    println!();
    let inflated = inflate(&deflated)?;
    println!("s:{}", String::from_utf8_lossy(&inflated).to_string());
    Ok(())
}
