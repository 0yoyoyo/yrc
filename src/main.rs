use std::env;
use std::str;
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

struct Tokens {
    list: Vec<Token>,
    current: usize,
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

fn tokenize(formula: &str) -> Result<Vec<Token>, String> {
    let mut v: Vec<Token> = Vec::new();
    let mut num_tmp: Vec<u8> = Vec::new();
    let mut op_tmp: Vec<u8> = Vec::new();
    let mut index = 0;
    let bytes = formula.as_bytes();
    let len = bytes.len();

    while index < len {
        match bytes[index] {
            b'0'..=b'9' => {
                num_tmp.push(bytes[index]);
                if (index + 1 < len && 
                    !b"0123456789".contains(&bytes[index + 1])) ||
                   index + 1 == len {
                    let num = str::from_utf8(&num_tmp).unwrap()
                              .parse().expect("Cannot parse!");
                    v.push(TokenNum(num));
                    num_tmp.clear();
                }
            },
            b'+' | b'-' |
            b'*' | b'/' |
            b'(' | b')' => {
                op_tmp.push(bytes[index]);
                let op = str::from_utf8(&op_tmp).unwrap().to_string();
                v.push(TokenOp(op));
                op_tmp.clear();
            },
            b'<' | b'>' |
            b'=' | b'!' => {
                op_tmp.push(bytes[index]);
                if (index + 1 < len && 
                    !b"<>=!".contains(&bytes[index + 1])) ||
                   index + 1 == len {
                    let op = str::from_utf8(&op_tmp).unwrap().to_string();
                    v.push(TokenOp(op));
                    op_tmp.clear();
                }
            },
            b' ' | b'\t'| b'\n' => (),
            _ => return Err(format!("Cannot tokenize!")),
        }
        index += 1;
    }

    v.push(TokenEnd);

    Ok(v)
}

impl Token {
    fn get_num(&self) -> std::option::Option<u32> {
        match self {
            TokenNum(num) => Some(*num),
            _ => None,
        }
    }

    fn get_op(&self) -> std::option::Option<&str> {
        match self {
            TokenOp(op) => Some(op),
            _ => None,
        }
    }
}

impl Tokens {
    fn expect_num(&mut self) -> u32 {
        if let Some(num) = self.list[self.current].get_num() {
            self.current += 1;
            num
        } else {
            println!("Not a number!");
            std::process::exit(1);
        }
    }

    fn expect_op(&mut self, expect: &str) -> bool {
        if let Some(op) = self.list[self.current].get_op() {
            if op == expect {
                self.current += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn has_next(&self) -> bool {
        match self.list[self.current] {
            TokenEnd => false,
            _ => true,
        }
    }

    fn new(v: Vec<Token>) -> Tokens {
        Tokens {
            list: v,
            current: 0,
        }
    }
}

fn primary(tokens: &mut Tokens) -> Box<Node> {
    let node: Box<Node>;
    if tokens.expect_op("(") {
        node = expr(tokens);
        tokens.expect_op(")");
    } else {
        node = new_node_num(tokens.expect_num());
    }
    node
}

fn unary(tokens: &mut Tokens) -> Box<Node> {
    let node: Box<Node>;
    if tokens.expect_op("-") {
        node = new_node(NodeSub, new_node_num(0), primary(tokens));
    } else {
        node = primary(tokens);
    }
    node
}

fn mul(tokens: &mut Tokens) -> Box<Node> {
    let mut node = unary(tokens);
    while tokens.has_next() {
        if tokens.expect_op("*") {
            node = new_node(NodeMul, node, unary(tokens));
        } else if tokens.expect_op("/") {
            node = new_node(NodeDiv, node, unary(tokens));
        } else {
            break;
        }
    }
    node
}

fn add(tokens: &mut Tokens) -> Box<Node> {
    let mut node = mul(tokens);
    while tokens.has_next() {
        if tokens.expect_op("+") {
            node = new_node(NodeAdd, node, mul(tokens));
        } else if tokens.expect_op("-") {
            node = new_node(NodeSub, node, mul(tokens));
        } else {
            break;
        }
    }
    node
}

fn relational(tokens: &mut Tokens) -> Box<Node> {
    let mut node = add(tokens);
    while tokens.has_next() {
        if tokens.expect_op("<") {
            node = new_node(NodeGr, node, add(tokens));
        } else if tokens.expect_op("<=") {
            node = new_node(NodeGe, node, add(tokens));
        } else if tokens.expect_op(">") {
            node = new_node(NodeGr, add(tokens), node);
        } else if tokens.expect_op(">=") {
            node = new_node(NodeGe, add(tokens), node);
        } else {
            break;
        }
    }
    node
}

fn equality(tokens: &mut Tokens) -> Box<Node> {
    let mut node = relational(tokens);
    while tokens.has_next() {
        if tokens.expect_op("==") {
            node = new_node(NodeEq, node, relational(tokens));
        } else if tokens.expect_op("!=") {
            node = new_node(NodeNe, node, relational(tokens));
        } else {
            break;
        }
    }
    node
}

fn expr(tokens: &mut Tokens) -> Box<Node> {
    let node = equality(tokens);
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

fn make_output_dir() -> std::io::Result<()> {
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

fn do_generate_asm(tokens: &mut Tokens) -> std::io::Result<()> {
    make_output_dir()?;

    let mut f = File::create("output/tmp.s")?;
    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;

    let node = expr(tokens);
    assemble_node(&mut f, node)?;

    f.write_fmt(format_args!("    pop rax\n"))?;
    f.write_fmt(format_args!("    ret\n"))?;
    Ok(())
}

fn generate_asm(formula: &str) -> std::result::Result<(), String> {
    let mut tokens: Tokens;
    match tokenize(formula) {
        Ok(token_list) => {
            tokens = Tokens::new(token_list);
        },
        Err(e) => return Err(e),
    }
    match do_generate_asm(&mut tokens) {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Cannot generate assembly code!")),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
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
