mod token;
mod parse;
mod assemble;

use std::env;
use std::str;
use std::fs;
use std::fs::File;

use token::tokenize;
use token::Tokens;
use parse::program;
use assemble::assemble;

fn make_output_dir() -> Result<(), String> {
    match fs::create_dir("output") {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(format!("Cannot create directory!"))
            }
        },
    }
}

fn generate_asm(formula: &str) -> Result<(), String> {
    let token_list = tokenize(formula)?;
    let mut tokens = Tokens::new(token_list);

    let nodes = program(&mut tokens)
        .map_err(|e| format!("{}", e))?;

    make_output_dir()?;
    let mut f = File::create("output/tmp.s")
        .map_err(|_| format!("Cannot create file"))?;

    match assemble(&mut f, nodes) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Cannot generate assembly code! {:?}", e))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        println!("{}", args[1]);
        match generate_asm(&args[1]) {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        };
    } else {
        println!("Invalid number of arguments!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn return_val_num(formula: &str, expect: u32) {
        assert_eq!(Ok(()), generate_asm(formula));
        let out = Command::new("bash")
                          .arg("-c")
                          .arg("script/test.sh")
                          .output()
                          .expect("Exec error!");
        let answer = String::from_utf8(out.stdout).unwrap();
        println!("{} -> {} (expected: {})",
                 formula, answer, expect);
        assert_eq!(format!("{}", expect), answer);
    }

    #[test]
    fn calc_unary() {
        return_val_num("0;", 0);
        return_val_num("123;", 123);
        //return_val_num("-123;", -123);
        return_val_num("(123);", 123);
        //return_val_num("-(123);", -123);
    }

    #[test]
    fn calc_binary() {
        return_val_num("1 + 2;", 3);
        return_val_num("3 - 2;", 1);
        return_val_num("2 * 3;", 6);
        return_val_num("6 / 2;", 3);
        return_val_num("7 == 7;", 1);
        return_val_num("7 == 8;", 0);
        return_val_num("7 != 7;", 0);
        return_val_num("7 != 8;", 1);
        return_val_num("7 < 8;", 1);
        return_val_num("7 <= 7;", 1);
        return_val_num("7 <= 8;", 1);
        return_val_num("7 < 7;", 0);
        return_val_num("7 <= 6;", 0);
        return_val_num("7 <= 6;", 0);
        return_val_num("8 > 7;", 1);
        return_val_num("7 >= 7;", 1);
        return_val_num("8 >= 7;", 1);
        return_val_num("7 > 7;", 0);
        return_val_num("6 >= 7;", 0);
        return_val_num("6 >= 7;", 0);
    }

    #[test]
    fn calc_combination() {
        return_val_num("-1 + 2;", 1);
        return_val_num("-(1 + 2) + 4;", 1);
        return_val_num("2 * 3 + 6 / 2;", 9);
        return_val_num("2 * (3 + 6) / 3;", 6);
    }

    #[test]
    fn calc_local_variable() {
        return_val_num("a = 1; a;", 1);
        return_val_num("z = 1; z;", 1);
        return_val_num("n = 10 + 2; n * 2;", 24);
    }

    #[test]
    fn check_format() {
        return_val_num("1+2+3;", 6);
        return_val_num(" 1 + 2 + 3 ;", 6);
        return_val_num("1 +  2   +    3;", 6);
        return_val_num("(1+2)+3;", 6);
        return_val_num("1+(2+3);", 6);
        return_val_num("(1+2+3);", 6);
    }
}
