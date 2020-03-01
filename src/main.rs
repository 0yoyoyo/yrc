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

const OUTPUT_DIR: & str = "output";

#[derive(Debug)]
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
    match fs::create_dir(OUTPUT_DIR) {
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
    let mut f = File::create(OUTPUT_DIR.to_string() + "/tmp.s")?;

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

    fn check_return_num(formula: &str, expect: u32) {
        compile(formula).unwrap();
        let output = Command::new("bash")
            .arg("-c")
            .arg("script/test.sh")
            .output()
            .unwrap();
        let answer = str::from_utf8(&output.stdout)
            .unwrap()
            .parse()
            .unwrap();
        println!("{} -> {} (expected: {})", formula, answer, expect);
        assert_eq!(expect, answer);
    }

    #[test]
    fn calc_unary() {
        check_return_num("0;", 0);
        check_return_num("123;", 123);
        //check_return_num("-123;", -123);
        check_return_num("(123);", 123);
        //check_return_num("-(123);", -123);
    }

    #[test]
    fn calc_binary() {
        check_return_num("1 + 2;", 3);
        check_return_num("3 - 2;", 1);
        check_return_num("2 * 3;", 6);
        check_return_num("6 / 2;", 3);
        check_return_num("7 == 7;", 1);
        check_return_num("7 == 8;", 0);
        check_return_num("7 != 7;", 0);
        check_return_num("7 != 8;", 1);
        check_return_num("7 < 8;", 1);
        check_return_num("7 <= 7;", 1);
        check_return_num("7 <= 8;", 1);
        check_return_num("7 < 7;", 0);
        check_return_num("7 <= 6;", 0);
        check_return_num("7 <= 6;", 0);
        check_return_num("8 > 7;", 1);
        check_return_num("7 >= 7;", 1);
        check_return_num("8 >= 7;", 1);
        check_return_num("7 > 7;", 0);
        check_return_num("6 >= 7;", 0);
        check_return_num("6 >= 7;", 0);
    }

    #[test]
    fn calc_combination() {
        check_return_num("-1 + 2;", 1);
        check_return_num("-(1 + 2) + 4;", 1);
        check_return_num("2 * 3 + 6 / 2;", 9);
        check_return_num("2 * (3 + 6) / 3;", 6);
    }

    #[test]
    fn calc_local_variable() {
        check_return_num("a = 1; a;", 1);
        check_return_num("z = 1; z;", 1);
        check_return_num("n = 10 + 2; n * 2;", 24);
    }

    #[test]
    fn check_format() {
        check_return_num("1+2+3;", 6);
        check_return_num(" 1 + 2 + 3 ;", 6);
        check_return_num("1 +  2   +    3;", 6);
        check_return_num("(1+2)+3;", 6);
        check_return_num("1+(2+3);", 6);
        check_return_num("(1+2+3);", 6);
    }
}
