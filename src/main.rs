use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    if args.len() == 2 {
        match generate_asm(&args[1]) {
            Ok(_) => (),
            Err(_) => println!("Failed!"),
        };
    } else {
        println!("Invalid!");
    }
}

fn generate_asm(formula: &str) -> std::io::Result<()> {
    match fs::create_dir("output") {
        _ => (),
    };
    let mut formula = formula.split_whitespace();
    let mut f = File::create("output/tmp.s")?;
    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;
    f.write_fmt(format_args!("    mov rax, {}\n",
                             formula.next().unwrap()))?;

    loop {
        match formula.next() {
            Some(item) => {
                if item == "+" {
                    f.write_fmt(format_args!("    add rax, {}\n",
                                             formula.next().unwrap()))?;
                } else if item == "-" {
                    f.write_fmt(format_args!("    sub rax, {}\n",
                                             formula.next().unwrap()))?;
                }
            }
            None => break,
        }
    }

    f.write_fmt(format_args!("    ret\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn return_val_num(formula: &str) {
        assert_eq!((), generate_asm(formula).unwrap());
        let out = Command::new("bash")
                          .arg("-c")
                          .arg("script/test.sh")
                          .output()
                          .expect("Exec error!");
        assert_eq!(format!("{}\n", formula),
                   String::from_utf8(out.stdout).unwrap());
    }

    #[test]
    fn return_val_0() {
        return_val_num("0");
    }

    #[test]
    fn return_val_123() {
        return_val_num("123");
    }
}
