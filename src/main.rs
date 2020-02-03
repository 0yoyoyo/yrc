use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    if args.len() == 2 {
        match fs::create_dir("output") {
            _ => (),
        };
        let num = &args[1];
        let mut f = File::create("output/num.txt")?;
        f.write_fmt(format_args!("{}", num))?;
    } else {
        println!("Invalid!");
    }
    Ok(())
}
