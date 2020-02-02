use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    let num = &args[1];
    println!("num: {}", num);

    let mut f = File::create("num.txt")?;
    f.write_fmt(format_args!("{}", num))?;
    Ok(())
}
