mod token;
mod parse;
mod assembly;

use std::env;
use std::str;
use std::fmt;
use std::io;
use std::fs;
use std::fs::File;

use token::tokenize;
use token::Tokens;
use token::TokenError;
use parse::Parser;
use parse::ParseError;
use assembly::AsmGenerator;
use assembly::AsmError;

use CompileError::*;

const OUTPUT_DIR: &str = "output";

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

fn compile_to_fname(formula: &str, fname: &str) -> Result<(), CompileError> {
    let token_list = tokenize(formula)?;
    let mut tokens = Tokens::new(token_list);

    let mut parser = Parser::new();
    let nodes = parser.program(&mut tokens)?;

    make_output_dir()?;
    let mut f = File::create(format!("{}/{}.s", OUTPUT_DIR, fname))?;

    let mut generator = AsmGenerator::new();
    generator.gen_asm(&mut f, nodes)?;

    Ok(())
}

fn compile(formula: &str) -> Result<(), CompileError> {
    compile_to_fname(formula, "tmp")
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
    use rand::prelude::*;

    fn random_string(len: usize) -> String {
        let source = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                       abcdefghijklmnopqrstuvwxyz\
                       0123456789";
        let mut rng = rand::thread_rng();

        String::from_utf8(
            source.choose_multiple(&mut rng, len)
                .cloned()
                .collect()
        ).unwrap()
    }

    fn check_return_num(formula: &str, expect: u32) {
        let fname = format!("tmp{}", random_string(8));
        compile_to_fname(formula, &fname).unwrap();

        let output = Command::new("bash")
            .arg("-c")
            .arg(format!("script/assemble.sh {}", fname))
            .output()
            .unwrap();
        let answer = str::from_utf8(&output.stdout)
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        fs::remove_file(format!("{}/{}.s", OUTPUT_DIR, fname)).unwrap();
        fs::remove_file(format!("{}/{}", OUTPUT_DIR, fname)).unwrap();

        println!("{} -> {} (expected: {})", formula, answer, expect);
        assert_eq!(expect, answer);
    }

    #[test]
    fn calc_unary() {
        check_return_num("fn main() { return 0; }", 0);
        check_return_num("fn main() { return 123; }", 123);
        check_return_num("fn main() { return (123); }", 123);
    }

    #[test]
    fn calc_binary() {
        check_return_num("fn main() { return 1 + 2; }", 3);
        check_return_num("fn main() { return 3 - 2; }", 1);
        check_return_num("fn main() { return 2 * 3; }", 6);
        check_return_num("fn main() { return 6 / 2; }", 3);
        check_return_num("fn main() { return 7 == 7; }", 1);
        check_return_num("fn main() { return 7 == 8; }", 0);
        check_return_num("fn main() { return 7 != 7; }", 0);
        check_return_num("fn main() { return 7 != 8; }", 1);
        check_return_num("fn main() { return 7 < 8; }", 1);
        check_return_num("fn main() { return 7 <= 7; }", 1);
        check_return_num("fn main() { return 7 <= 8; }", 1);
        check_return_num("fn main() { return 7 < 7; }", 0);
        check_return_num("fn main() { return 7 <= 6; }", 0);
        check_return_num("fn main() { return 7 <= 6; }", 0);
        check_return_num("fn main() { return 8 > 7; }", 1);
        check_return_num("fn main() { return 7 >= 7; }", 1);
        check_return_num("fn main() { return 8 >= 7; }", 1);
        check_return_num("fn main() { return 7 > 7; }", 0);
        check_return_num("fn main() { return 6 >= 7; }", 0);
        check_return_num("fn main() { return 6 >= 7; }", 0);
    }

    #[test]
    fn calc_combination() {
        check_return_num("fn main() { return -1 + 2; }", 1);
        check_return_num("fn main() { return -(1 + 2) + 4; }", 1);
        check_return_num("fn main() { return 2 * 3 + 6 / 2; }", 9);
        check_return_num("fn main() { return 2 * (3 + 6) / 3; }", 6);
    }

    #[test]
    fn calc_local_variable() {
        check_return_num("fn main() {\
                              let a: i32;\
                              a = 1;\
                              return a;\
                          }", 1);
        check_return_num("fn main() {\
                              let z: i32;\
                              z = 1;\
                              return z;\
                          }", 1);
        check_return_num("fn main() {\
                              let n: i32;\
                              n = 10 + 2;\
                              return n * 2;\
                          }", 24);
        check_return_num("fn main() {\
                              let abc: i32;\
                              let def: i32;\
                              abc = 2;\
                              def = 3;\
                              return abc + def;\
                          }", 5);
        check_return_num("fn main() {\
                              let abc: i32;\
                              let def: i32;\
                              abc = 2;\
                              def = abc + 3;\
                              return def;\
                          }", 5);
    }

    #[test]
    fn calc_control() {
        check_return_num("fn main() {\
                              let a: i32;\
                              a = 1;\
                              if 1 == 1 {\
                                  a = 2;\
                              } else {\
                                  a = 3;\
                              }\
                              return a;\
                          }", 2);
        check_return_num("fn main() {\
                              let a: i32;\
                              a = 1;\
                              if 1 == 2 {\
                                  a = 2;\
                              } else {\
                                  a = 3;\
                              }\
                              return a;\
                          }", 3);
        check_return_num("fn main() {\
                              let a: i32;\
                              let b: i32;\
                              a = 1;\
                              if 1 == 1 {\
                                  b = 1;\
                                  a = b + 1;\
                              }\
                              return a;\
                          }", 2);
        check_return_num("fn main() {\
                              let a: i32;\
                              a = 1;\
                              if 1 == 1 {\
                                  a = a + 1;\
                              }\
                              if 1 == 2 {\
                                  a = a + 1;\
                              }\
                              if 2 == 2 {\
                                  a = a + 1;\
                                  if 3 == 3 {\
                                      a = a + 1;\
                                  }\
                              }\
                              return a;\
                          }", 4);
        check_return_num("fn main() {\
                              let a: i32;\
                              a = 1;\
                              while a != 10 {\
                                  a = a + 1;\
                              }\
                              return a;\
                          }", 10);
    }

    #[test]
    fn calc_func() {
        check_return_num("fn foo() {\
                              return 3;\
                          }\
                          fn main() {\
                              return foo();\
                          }", 3);
        check_return_num("fn foo() {\
                              let c: i32;\
                              let d: i32;\
                              c = 3;\
                              d = 4;\
                              return c + d;\
                          }\
                          fn main() {\
                              let a: i32;\
                              let b: i32;\
                              a = 1;\
                              b = 2;\
                              return a + b + foo();\
                          }", 10);
        check_return_num("fn foo() {\
                              let a: i32;\
                              let b: i32;\
                              a = 3;\
                              b = 4;\
                              return a + b;\
                          }\
                          fn main() {\
                              let a: i32;\
                              let b: i32;\
                              a = 1;\
                              b = 2;\
                              return a + b + foo();\
                          }", 10);
        check_return_num("fn foo(a: i32) {\
                              return a * 2;\
                          }\
                          fn main() {\
                              return foo(2);\
                          }", 4);
        check_return_num("fn foo(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) {\
                              return (a + b + c + d + e + f) * 2;\
                          }\
                          fn main() {\
                              return foo(1, 2, 3, 4, 5, 6);\
                          }", 42);
    }

    #[test]
    fn calc_reference() {
        check_return_num("fn main() {\
                              let a: i32;\
                              let b: i32;\
                              a = 2;\
                              b = &a;
                              return *b;\
                          }", 2);
    }

    #[test]
    fn check_format() {
        check_return_num("fn main() { return 1+2+3; }", 6);
        check_return_num("fn main() { return  1 + 2 + 3 ; }", 6);
        check_return_num("fn main() { return 1 +  2   +    3; }", 6);
        check_return_num("fn main() { return (1+2)+3; }", 6);
        check_return_num("fn main() { return 1+(2+3); }", 6);
        check_return_num("fn main() { return (1+2+3); }", 6);
    }
}
