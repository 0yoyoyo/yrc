use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use NodeKind::*;
use Token::*;

#[derive(PartialEq)]
enum Token {
    TokenOp(String),
    TokenNum(u32),
    TokenEnd,
}

enum NodeKind {
    NodeAdd,
    NodeSub,
}

enum Node {
    Operator {
        kind: NodeKind,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Number {
        val: u32,
    },
}

fn new_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Box<Node> {
    let node = Node::Operator {
        kind: kind,
        lhs: lhs,
        rhs: rhs,
    };
    let node = Box::new(node);
    node
}

fn new_node_num(val: u32) -> Box<Node> {
    let node = Node::Number {
        val: val,
    };
    let node = Box::new(node);
    node
}

fn tokenize(formula: &str) -> Vec<Token> {
    let mut v: Vec<Token> = Vec::new();
    let mut num_tmp = String::new();
    for c in formula.chars() {
        if c.is_ascii_digit() {
            num_tmp.push(c);
        } else if !num_tmp.is_empty() {
            v.push(TokenNum(num_tmp.parse().expect("Cannot parse!")));
            num_tmp.clear();
        }

        if c == '+' || c == '-' {
            v.push(TokenOp(c.to_string()));
        }
    }
    if !num_tmp.is_empty() {
        v.push(TokenNum(num_tmp.parse().expect("Cannot parse!")));
        num_tmp.clear();
    }
    v.push(TokenEnd);
    v
}

fn expect_num(v: &mut Vec<Token>) -> u32 {
    let number: u32;
    match &v[0] {
        TokenNum(num) => number = *num,
        _ => unreachable!(),
    }
    &v.remove(0);
    number
}

fn expect_op(v: &mut Vec<Token>) -> String {
    let operator: String;
    match &v[0] {
        TokenOp(op) => operator = op.to_string(),
        _ => unreachable!(),
    }
    &v.remove(0);
    operator
}

fn expr(v: &mut Vec<Token>) -> Box<Node> {
    let mut node = new_node_num(expect_num(v));
    while v[0] != TokenEnd {
        match expect_op(v).as_str() {
            "+" => {
                node = new_node(NodeAdd,
                                node,
                                new_node_num(expect_num(v)));
            },
            "-" => {
                node = new_node(NodeSub,
                                node,
                                new_node_num(expect_num(v)));
            },
            _ => unreachable!(),
        }
    }
    node
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
            }
            f.write_fmt(format_args!("    push rax\n"))?;
        },
    }
    Ok(())
}

fn generate_asm(formula: &str) -> std::io::Result<()> {
    match fs::create_dir("output") {
        _ => (),
    };
    let mut f = File::create("output/tmp.s")?;
    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;

    let mut v = tokenize(formula);
    let node = expr(&mut v);
    assemble_node(&mut f, node)?;

    f.write_fmt(format_args!("    pop rax\n"))?;
    f.write_fmt(format_args!("    ret\n"))?;
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn return_val_num(formula: &str, answer: u32) {
        assert_eq!((), generate_asm(formula).unwrap());
        let out = Command::new("bash")
                          .arg("-c")
                          .arg("script/test.sh")
                          .output()
                          .expect("Exec error!");
        assert_eq!(format!("{}\n", answer),
                   String::from_utf8(out.stdout).unwrap());
    }

    #[test]
    fn return_val_0() {
        return_val_num("0", 0);
    }

    #[test]
    fn return_val_123() {
        return_val_num("123", 123);
    }

    #[test]
    fn return_val_formula_with_space() {
        return_val_num(" 123 + 23 - 6 ", 140);
    }

    #[test]
    fn return_val_formula_without_space() {
        return_val_num("123+23-6", 140);
    }
}
