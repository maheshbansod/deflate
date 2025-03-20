use anyhow::Result;
use deflate::inflate;

fn main() -> Result<()> {
    let f = include_bytes!("../test_data/nocompression-deflate2.deflate");
    let inflated = inflate(f)?;
    println!("s:{}", String::from_utf8_lossy(&inflated).to_string());
    // let d = deflate(f.);
    // println!("{:?}", d);
    Ok(())
}
