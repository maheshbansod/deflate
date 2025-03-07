use anyhow::Result;

use deflate::deflate;

fn main() -> Result<()> {
    let d = deflate(b"abc");
    println!("{:?}", d);
    Ok(())
}
