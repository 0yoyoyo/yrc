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
        let mut f = File::create("output/tmp.s")?;
        f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
        f.write_fmt(format_args!(".global main\n"))?;
        f.write_fmt(format_args!("main:\n"))?;
        f.write_fmt(format_args!("    mov rax, {}\n", num))?;
        f.write_fmt(format_args!("    ret\n"))?;
    } else {
        println!("Invalid!");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //use super::*;
    use std::process::Command;

    #[test]
    fn return_val_0() {
        let out = Command::new("bash")
                          .arg("-c")
                          .arg("script/test.sh 0 0")
                          .output()
                          .expect("Exec error!");
        assert_eq!(b"0\n", out.stdout.as_slice());
    }
}
