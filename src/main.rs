use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    if args.len() == 2 {
        match generate_asm(args[1].parse().unwrap()){
            Ok(_) => (),
            Err(_) => println!("Failed!"),
        };
    } else {
        println!("Invalid!");
    }
}

fn generate_asm(num: u32) -> std::io::Result<()> {
    match fs::create_dir("output") {
        _ => (),
    };
    let mut f = File::create("output/tmp.s")?;
    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;
    f.write_fmt(format_args!("    mov rax, {}\n", num))?;
    f.write_fmt(format_args!("    ret\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn return_val_num(n: u32) {
        assert_eq!((), generate_asm(n).unwrap());
        let out = Command::new("bash")
                          .arg("-c")
                          .arg("script/test.sh 0 0")
                          .output()
                          .expect("Exec error!");
        assert_eq!(format!("{}\n", n),
                   String::from_utf8(out.stdout).unwrap());
    }

    #[test]
    fn return_val_0() {
        return_val_num(0);
    }

    #[test]
    fn return_val_123() {
        return_val_num(123);
    }
}
