use anyhow::Result;
use deflate::inflate;

fn main() -> Result<()> {
    let correct_text = "Some text. Oh text.. oh text oh text ooh.. yayyyy ayayayya.
    I am a text fan aaaahhh.. I\"m a text fan. I need just text. Gimme the text! Gimme.";
    println!("original:{correct_text}");

    let f = include_bytes!("../test_data/fixed-comp-deflate.deflate");
    println!("to inflate");
    for b in f {
        print!("{b:0>8b} ");
    }
    println!("inflating");
    let inflated = inflate(f)?;
    let inflated_s = String::from_utf8_lossy(&inflated).to_string();
    println!("\ninflated: s:\n{}", inflated_s);

    println!("Are both equal?");
    if correct_text == inflated_s {
        println!("Yes!!");
    } else {
        println!("Nah, look again.");
    }

    Ok(())
}
