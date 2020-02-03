use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    if args.len() == 2 {
        let num = &args[1];
        let mut f = File::create("num.txt")?;
        f.write_fmt(format_args!("{}", num))?;
    } else {
        println!("Invalid!");
    }
    Ok(())
}
