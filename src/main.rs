mod token;
mod parse;

use std::env;
use std::str;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use NodeKind::*;
use token::tokenize;
use token::Tokens;
use parse::Node;
use parse::NodeKind;
use parse::expr;

fn make_output_dir() -> std::result::Result<(), String> {
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

fn assemble_node(f: &mut File, node: Box<Node>) -> std::io::Result<()> {
    match *node {
        Node::Number {val} => {
            f.write_fmt(format_args!("    push {}\n", val))?;
        },
        Node::Operator { kind, lhs, rhs} => {
            assemble_node(f, lhs)?;
            assemble_node(f, rhs)?;
            f.write_fmt(format_args!("    pop rdi\n"))?;
            f.write_fmt(format_args!("    pop rax\n"))?;
            match kind {
                NodeAdd => {
                    f.write_fmt(format_args!("    add rax, rdi\n"))?;
                },
                NodeSub => {
                    f.write_fmt(format_args!("    sub rax, rdi\n"))?;
                },
                NodeMul => {
                    f.write_fmt(format_args!("    imul rax, rdi\n"))?;
                },
                NodeDiv => {
                    f.write_fmt(format_args!("    cqo\n"))?;
                    f.write_fmt(format_args!("    idiv rdi\n"))?;
                },
                NodeEq => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    sete al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
                NodeNe => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    setne al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
                NodeGr => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    setl al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
                NodeGe => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    setle al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
            }
            f.write_fmt(format_args!("    push rax\n"))?;
        },
    }

    Ok(())
}

fn assemble(node: Box<Node>) -> std::io::Result<()> {
    let mut f = File::create("output/tmp.s")?;

    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;

    assemble_node(&mut f, node)?;

    f.write_fmt(format_args!("    pop rax\n"))?;
    f.write_fmt(format_args!("    ret\n"))?;

    Ok(())
}

fn generate_asm(formula: &str) -> std::result::Result<(), String> {
    make_output_dir()?;

    let token_list = tokenize(formula)?;
    let mut tokens = Tokens::new(token_list);

    let node = expr(&mut tokens);
    if tokens.has_next() {
        return Err(format!("Redundant numbers!"));
    }

    match assemble(node) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Cannot generate assembly code!")),
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
        return_val_num("0", 0);
        return_val_num("123", 123);
        //return_val_num("-123", -123);
        return_val_num("(123)", 123);
        //return_val_num("-(123)", -123);
    }

    #[test]
    fn calc_binary() {
        return_val_num("1 + 2", 3);
        return_val_num("3 - 2", 1);
        return_val_num("2 * 3", 6);
        return_val_num("6 / 2", 3);
        return_val_num("7 == 7", 1);
        return_val_num("7 == 8", 0);
        return_val_num("7 != 7", 0);
        return_val_num("7 != 8", 1);
        return_val_num("7 < 8", 1);
        return_val_num("7 <= 7", 1);
        return_val_num("7 <= 8", 1);
        return_val_num("7 < 7", 0);
        return_val_num("7 <= 6", 0);
        return_val_num("7 <= 6", 0);
        return_val_num("8 > 7", 1);
        return_val_num("7 >= 7", 1);
        return_val_num("8 >= 7", 1);
        return_val_num("7 > 7", 0);
        return_val_num("6 >= 7", 0);
        return_val_num("6 >= 7", 0);
    }

    #[test]
    fn calc_combination() {
        return_val_num("-1 + 2", 1);
        return_val_num("-(1 + 2) + 4", 1);
        return_val_num("2 * 3 + 6 / 2", 9);
        return_val_num("2 * (3 + 6) / 3", 6);
    }

    #[test]
    fn check_format() {
        return_val_num("1+2+3", 6);
        return_val_num(" 1 + 2 + 3 ", 6);
        return_val_num("1 +  2   +    3", 6);
        return_val_num("(1+2)+3", 6);
        return_val_num("1+(2+3)", 6);
        return_val_num("(1+2+3)", 6);
    }
}
