use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

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

fn expr(formula: &str) -> Box<Node> {
    let mut formula = formula.split_whitespace();

    let mut node = new_node_num(formula.next().unwrap().parse().unwrap());
    loop {
        match formula.next() {
            Some(item) => {
                if item == "+" {
                    node = new_node(NodeKind::NodeAdd,
                                    node,
                                    new_node_num(formula.next().unwrap().parse().unwrap()));
                } else if item == "-" {
                    node = new_node(NodeKind::NodeSub,
                                    node,
                                    new_node_num(formula.next().unwrap().parse().unwrap()));
                }
            }
            None => break,
        }
    }
    node
}

fn gen(f: &mut File, node: Box<Node>) {
    match *node {
        Node::Number {val} => {
            f.write_fmt(format_args!("    push {}\n", val)).unwrap()
        },
        Node::Operator { kind, lhs, rhs} => {
            gen(f, lhs);
            gen(f, rhs);
            f.write_fmt(format_args!("    pop rdi\n")).unwrap();
            f.write_fmt(format_args!("    pop rax\n")).unwrap();
            match kind {
                NodeKind::NodeAdd => {
                    f.write_fmt(format_args!("    add rax, rdi\n")).unwrap();
                },
                NodeKind::NodeSub => {
                    f.write_fmt(format_args!("    sub rax, rdi\n")).unwrap();
                },
            }
            f.write_fmt(format_args!("    push rax\n")).unwrap()
        },
    }
}

fn generate_asm(formula: &str) -> std::io::Result<()> {
    match fs::create_dir("output") {
        _ => (),
    };
    let mut f = File::create("output/tmp.s")?;
    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;

    let node = expr(formula);
    gen(&mut f, node);

    f.write_fmt(format_args!("    pop rax\n"))?;
    f.write_fmt(format_args!("    ret\n"))?;
    Ok(())
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
    fn return_val_formula() {
        return_val_num("123 + 23 - 6", 140);
    }
}
