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
    NodeMul,
    NodeDiv,
    NodeEq,
    NodeNe,
    NodeGr,
    NodeGe,
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
    let mut op_tmp = String::new();
    for c in formula.chars() {
        if c.is_ascii_digit() {
            num_tmp.push(c);
        } else if !num_tmp.is_empty() {
            let num = num_tmp.parse().expect("Cannot parse!");
            v.push(TokenNum(num));
            num_tmp.clear();
        }

        if c == '+' || c == '-' ||
           c == '*' || c == '/' ||
           c == '<' || c == '>' ||
           c == '=' || c == '!' {
            op_tmp.push(c);
        } else if c == '(' || c == ')' {
            if !op_tmp.is_empty() {
                let op = op_tmp.to_string();
                v.push(TokenOp(op));
                op_tmp.clear();
            }
            v.push(TokenOp(c.to_string()));
        } else if !op_tmp.is_empty() {
            let op = op_tmp.to_string();
            v.push(TokenOp(op));
            op_tmp.clear();
        }
    }
    if !num_tmp.is_empty() {
        let num = num_tmp.parse().expect("Cannot parse!");
        v.push(TokenNum(num));
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

fn expect_op(v: &mut Vec<Token>, expect: &str) -> bool {
    match &v[0] {
        TokenOp(op) => {
            if op == expect {
                &v.remove(0);
                true
            } else {
                false
            }
        },
        _ => false,
    }
}

fn primary(v: &mut Vec<Token>) -> Box<Node> {
    let node: Box<Node>;
    if expect_op(v, "(") {
        node = expr(v);
        expect_op(v, ")");
    } else {
        node = new_node_num(expect_num(v));
    }
    node
}

fn unary(v: &mut Vec<Token>) -> Box<Node> {
    let node: Box<Node>;
    if expect_op(v, "-") {
        node = new_node(NodeSub, new_node_num(0), primary(v));
    } else {
        node = primary(v);
    }
    node
}

fn mul(v: &mut Vec<Token>) -> Box<Node> {
    let mut node = unary(v);
    while v[0] != TokenEnd {
        if expect_op(v, "*") {
            node = new_node(NodeMul, node, unary(v));
        } else if expect_op(v, "/") {
            node = new_node(NodeDiv, node, unary(v));
        } else {
            break;
        }
    }
    node
}

fn add(v: &mut Vec<Token>) -> Box<Node> {
    let mut node = mul(v);
    while v[0] != TokenEnd {
        if expect_op(v, "+") {
            node = new_node(NodeAdd, node, mul(v));
        } else if expect_op(v, "-") {
            node = new_node(NodeSub, node, mul(v));
        } else {
            break;
        }
    }
    node
}

fn relational(v: &mut Vec<Token>) -> Box<Node> {
    let mut node = add(v);
    while v[0] != TokenEnd {
        if expect_op(v, "<") {
            node = new_node(NodeGr, node, add(v));
        } else if expect_op(v, "<=") {
            node = new_node(NodeGe, node, add(v));
        } else if expect_op(v, ">") {
            node = new_node(NodeGr, add(v), node);
        } else if expect_op(v, ">=") {
            node = new_node(NodeGe, add(v), node);
        } else {
            break;
        }
    }
    node
}

fn equality(v: &mut Vec<Token>) -> Box<Node> {
    let mut node = relational(v);
    while v[0] != TokenEnd {
        if expect_op(v, "==") {
            node = new_node(NodeEq, node, relational(v));
        } else if expect_op(v, "!=") {
            node = new_node(NodeNe, node, relational(v));
        } else {
            break;
        }
    }
    node
}

fn expr(v: &mut Vec<Token>) -> Box<Node> {
    let node = equality(v);
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
