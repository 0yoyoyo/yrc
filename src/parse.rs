use crate::token::Tokens;

use NodeKind::*;

#[derive(PartialEq)]
pub enum NodeKind {
    NodeAdd,
    NodeSub,
    NodeMul,
    NodeDiv,
    NodeEq,
    NodeNe,
    NodeGr,
    NodeGe,
    NodeAsn,
}

pub enum Node {
    Operator {
        kind: NodeKind,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Number {
        val: u32,
    },
    LocalVariable {
        offset: usize,
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

fn new_node_var(var: &str) -> Box<Node> {
    let node = Node::LocalVariable {
        offset: ((var.as_bytes()[0] - b'a' + 1) * 8) as usize,
    };
    let node = Box::new(node);
    node
}

fn primary(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let node: Box<Node>;
    if tokens.expect_op("(") {
        match expr(tokens) {
            Ok(n) => node = n,
            Err(e) => return Err(e),
        }
        if !tokens.expect_op(")") {
            return Err(format!("Cannot find right parenthesis!"));
        }
    } else if let Some(var) = tokens.expect_var() {
        node = new_node_var(var);
    } else {
        let num = match tokens.expect_num() {
            Some(n) => n,
            None => {
                print!("{}^ ", " ".repeat(tokens.head()));
                return Err(format!("Not a number!"));
            },
        };
        node = new_node_num(num);
    }
    Ok(node)
}

fn unary(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    if tokens.expect_op("-") {
        primary(tokens).map(|rhs| new_node(NodeSub, new_node_num(0), rhs))
    } else {
        primary(tokens)
    }
}

fn mul(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node: Box<Node>;
    match unary(tokens) {
        Ok(n) => node = n,
        Err(e) => return Err(e),
    }
    while tokens.has_next() {
        if tokens.expect_op("*") {
            match unary(tokens) {
                Ok(rhs) => node = new_node(NodeMul, node, rhs),
                Err(e) => return Err(e),
            }
        } else if tokens.expect_op("/") {
            match unary(tokens) {
                Ok(rhs) => node = new_node(NodeDiv, node, rhs),
                Err(e) => return Err(e),
            }
        } else {
            break;
        }
    }
    Ok(node)
}

fn add(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node: Box<Node>;
    match mul(tokens) {
        Ok(n) => node = n,
        Err(e) => return Err(e),
    }
    while tokens.has_next() {
        if tokens.expect_op("+") {
            match mul(tokens) {
                Ok(rhs) => node = new_node(NodeAdd, node, rhs),
                Err(e) => return Err(e),
            }
        } else if tokens.expect_op("-") {
            match mul(tokens) {
                Ok(rhs) => node = new_node(NodeSub, node, rhs),
                Err(e) => return Err(e),
            }
        } else {
            break;
        }
    }
    Ok(node)
}

fn relational(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node: Box<Node>;
    match add(tokens) {
        Ok(n) => node = n,
        Err(e) => return Err(e),
    }
    while tokens.has_next() {
        if tokens.expect_op("<") {
            match add(tokens) {
                Ok(rhs) => node = new_node(NodeGr, node, rhs),
                Err(e) => return Err(e),
            }
        } else if tokens.expect_op("<=") {
            match add(tokens) {
                Ok(rhs) => node = new_node(NodeGe, node, rhs),
                Err(e) => return Err(e),
            }
        } else if tokens.expect_op(">") {
            match add(tokens) {
                Ok(lhs) => node = new_node(NodeGr, lhs, node),
                Err(e) => return Err(e),
            }
        } else if tokens.expect_op(">=") {
            match add(tokens) {
                Ok(lhs) => node = new_node(NodeGe, lhs, node),
                Err(e) => return Err(e),
            }
        } else {
            break;
        }
    }
    Ok(node)
}

fn equality(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node: Box<Node>;
    match relational(tokens) {
        Ok(n) => node = n,
        Err(e) => return Err(e),
    }
    while tokens.has_next() {
        if tokens.expect_op("==") {
            match relational(tokens) {
                Ok(rhs) => node = new_node(NodeEq, node, rhs),
                Err(e) => return Err(e),
            }
        } else if tokens.expect_op("!=") {
            match relational(tokens) {
                Ok(rhs) => node = new_node(NodeNe, node, rhs),
                Err(e) => return Err(e),
            }
        } else {
            break;
        }
    }
    Ok(node)
}

fn assign(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    match equality(tokens) {
        Ok(node) => {
            if tokens.expect_op("=") {
                assign(tokens)
                    .map(|rhs| new_node(NodeAsn, node, rhs))
            } else {
                Ok(node)
            }
        },
        Err(e) => Err(e),
    }
}

fn expr(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    assign(tokens)
}

fn stmt(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    match expr(tokens) {
        Ok(node) => {
            if tokens.expect_op(";") {
                Ok(node)
            } else {
                Err(format!("Cannot find semicolon!"))
            }
        },
        Err(e) => Err(e),
    }
}

pub fn program(tokens: &mut Tokens) -> Result<Vec<Box<Node>>, String> {
    let mut nodes: Vec<Box<Node>> = Vec::new();
    while tokens.has_next() {
        stmt(tokens).map(|node| nodes.push(node))?;
    }
    Ok(nodes)
}
