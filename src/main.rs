mod token;
mod parse;
mod assemble;

use std::env;
use std::str;
use std::fmt;
use std::io;
use std::fs;
use std::fs::File;

use token::tokenize;
use token::Tokens;
use token::TokenError;
use parse::program;
use parse::ParseError;
use assemble::assemble;
use assemble::AsmError;

use CompileError::*;

enum CompileError {
    Env(io::Error),
    Token(TokenError),
    Parse(ParseError),
    Asm(AsmError),
}

impl From<io::Error> for CompileError {
    fn from(e: io::Error) -> Self {
        Env(e)
    }
}

impl From<TokenError> for CompileError {
    fn from(e: TokenError) -> Self {
        Token(e)
    }
}

impl From<ParseError> for CompileError {
    fn from(e: ParseError) -> Self {
        Parse(e)
    }
}

impl From<AsmError> for CompileError {
    fn from(e: AsmError) -> Self {
        Asm(e)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Env(e) => write!(f, "{}", e),
            Token(e) => write!(f, "{}", e),
            Parse(e) => write!(f, "{}", e),
            Asm(e) => write!(f, "{}", e),
        }
    }
}

fn make_output_dir() -> Result<(), io::Error> {
    match fs::create_dir("output") {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        },
    }
}

fn compile(formula: &str) -> Result<(), CompileError> {
    let token_list = tokenize(formula)?;
    let mut tokens = Tokens::new(token_list);

    let nodes = program(&mut tokens)?;

    make_output_dir()?;
    let mut f = File::create("output/tmp.s")?;

    assemble(&mut f, nodes)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        match compile(&args[1]) {
            Ok(_) => (),
            Err(e) => {
                println!("Error!");
                match e {
                    Env(e) => println!("{}", e),
                    _ => {
                        println!("{}", args[1]);
                        println!("{}", e);
                    },
                };
            },
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
        assert_eq!(Ok(()), compile(formula));
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
