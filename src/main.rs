mod token;
mod parse;
mod assembly;

use std::env;
use std::str;
use std::fmt;
use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use rand::prelude::*;
use getopts::Options;

use token::tokenize;
use token::Tokens;
use token::TokenError;
use parse::Parser;
use parse::ParseError;
use assembly::AsmGenerator;
use assembly::AsmError;

use CompileError::*;

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

fn compile_to_fname(formula: &str, fname: &str) -> Result<(), CompileError> {
    let token_list = tokenize(formula)?;
    let mut tokens = Tokens::new(token_list);

    let mut parser = Parser::new();
    let nodes = parser.program(&mut tokens)?;

    let mut f = File::create(format!("{}", fname))?;

    let literals = parser.literals();
    let mut generator = AsmGenerator::new();
    generator.gen_asm(&mut f, &nodes, literals)?;

    Ok(())
}

fn print_output(result: io::Result<Output>) {
    match result {
        Ok(output) => {
            print!("{}", str::from_utf8(&output.stdout).unwrap());
            print!("{}", str::from_utf8(&output.stderr).unwrap());
        },
        Err(e) => {
            println!("{}", e);
        },
    }
}

fn cmd_assemble(src: &str, dst: &str) {
    let cmd_result = Command::new("gcc")
        .arg(src)
        .arg("-o")
        .arg(dst)
        .output();

    print_output(cmd_result);
}

fn cmd_remove_asm(src: &str) {
    let cmd_result = Command::new("rm")
        .arg("-f")
        .arg(src)
        .output();

    print_output(cmd_result);
}

fn cmd_rename_asm(src: &str, dst: &str) {
    let cmd_result = Command::new("mv")
        .arg(src)
        .arg(dst)
        .output();

    print_output(cmd_result);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Input file is needed!");
        return;
    }

    let mut opts = Options::new();
    opts.optopt("o", "output", "set output file name", "NAME");
    opts.optflag("s", "asm", "output assemble code");
    opts.optflag("h", "help", "print this help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("Invalid option!");
            return;
        },
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(""));
        return;
    }
    let asm_out = matches.opt_present("s");
    let output_file = matches.opt_str("o");

    let input_file = match matches.free.get(0) {
        Some(s) => s,
        None => {
            println!("Input file is needed!");
            return;
        },
    };

    let path = Path::new(input_file);
    let default_name = path.file_stem().unwrap().to_str().unwrap();
    let output_file = match output_file {
        Some(s) => s,
        None => {
            if asm_out {
                let default_asm_name = format!("{}.s", default_name);
                default_asm_name.to_string()
            } else {
                default_name.to_string()
            }
        }
    };

    let source_code = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            return;
        },
    };

    let tmp_file = format!("tmp{}.s", random_string(8));

    match compile_to_fname(&source_code, &tmp_file) {
        Ok(_) => (),
        Err(e) => {
            println!("Error!");
            match e {
                Env(e) => println!("{}", e),
                _ => {
                    println!("{}", &source_code.replace("\n", " "));
                    println!("{}", e);
                    cmd_remove_asm(&tmp_file);
                    return;
                },
            };
        },
    };

    if !asm_out {
        cmd_assemble(&tmp_file, &output_file);
    } else {
        cmd_rename_asm(&tmp_file, &output_file);
    }
    cmd_remove_asm(&tmp_file);
}

#[cfg(test)]
mod tests {
    use super::*;

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

        fs::remove_file(format!("{}.s", fname)).unwrap();
        fs::remove_file(format!("{}", fname)).unwrap();

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
    fn calc_type() {
        check_return_num("fn main() {\
                              let a: i8;\
                              a = 1;\
                              return a;\
                          }", 1);
        check_return_num("fn main() {\
                              let a: i16;\
                              a = 1;\
                              return a;\
                          }", 1);
        check_return_num("fn main() {\
                              let a: i32;\
                              a = 1;\
                              return a;\
                          }", 1);
        check_return_num("fn main() {\
                              let a: i64;\
                              a = 1;\
                              return a;\
                          }", 1);
        // To check upper bits are cleared.
        check_return_num("static a: i8;\
                          fn main() {\
                              let b: i8;\
                              a = 1;\
                              b = 1;\
                              return a == b;\
                          }", 1);
        check_return_num("static a: i16;\
                          fn main() {\
                              let b: i16;\
                              a = 1;\
                              b = 1;\
                              return a == b;\
                          }", 1);
    }

    #[test]
    fn calc_global_variable() {
        check_return_num("static a: i32;\
                          fn main() {\
                              a = 1;\
                              return a;\
                          }", 1);
        check_return_num("static a: [i32; 10];\
                          fn main() {\
                              a[8] = 1;\
                              a[9] = 2;\
                              return a[8] + a[9];\
                          }", 3);
        check_return_num("static a: i32;\
                          fn main() {\
                              let b: i32;\
                              a = 1;\
                              b = 2;\
                              return a + b;\
                          }", 3);
        check_return_num("static a: [i32; 2];\
                          static b: [i32; 2];\
                          fn main() {\
                              a[1] = 1;\
                              b[0] = 2;\
                              return a[1] + b[0];\
                          }", 3);
        check_return_num("static a: [i8; 4];\
                          static b: i32;\
                          fn main() {\
                              b = 2;\
                              a[3] = 1;\
                              return a[3] + b;\
                          }", 3);
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
                              let b: &i32;\
                              a = 2;\
                              b = &a;\
                              return *b;\
                          }", 2);
        check_return_num("fn foo() {\
                              let a: i32;\
                              let b: &i32;\
                              b = &a;\
                              *b = 3;\
                              return a;\
                          }\
                          fn main() {\
                              return foo();\
                          }", 3);
    }

    #[test]
    fn calc_array() {
        check_return_num("fn main() {\
                              let a: [i32; 10];\
                              a[0] = 1;\
                              a[1] = 2;\
                              a[2] = 3;\
                              return a[0] + a[1] + a[2];\
                          }", 6);
        check_return_num("fn foo() {\
                              let a: [i32; 4];\
                              a[2] = 3;\
                              return a[2];\
                          }\
                          fn main() {\
                              return foo();\
                          }", 3);
        check_return_num("fn main() {\
                              let a: [i32; 4];\
                              let b: [i32; 4];\
                              a[3] = 2;\
                              b[3] = 3;\
                              return a[3] + b[3];\
                          }", 5);
    }

    #[test]
    fn check_comment() {
        check_return_num("fn main() {\
                              // This is\n\
                              // one line\n\
                              // comment.\n\
                              return 1;\
                          }", 1);
        check_return_num("fn main() {\
                              /*\
                               * This is\
                               * multiple line\
                               * comment.\
                               */\
                              return 1;\
                          }", 1);
        check_return_num("fn main() {\
                              /* No content */\
                              /**/\
                              return 1;\
                          }", 1);
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
