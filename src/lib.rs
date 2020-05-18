mod token;
mod parse;
mod assembly;

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

pub fn compiler_main(args: Vec<String>) {
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
