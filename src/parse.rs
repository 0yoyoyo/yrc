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

fn primary(tokens: &mut Tokens) -> Box<Node> {
    let node: Box<Node>;
    if tokens.expect_op("(") {
        node = expr(tokens);
        tokens.expect_op(")");
    } else if let Some(var) = tokens.expect_var() {
        node = new_node_var(var);
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

fn assign(tokens: &mut Tokens) -> Box<Node> {
    let mut node = equality(tokens);
    if tokens.expect_op("=") {
        node = new_node(NodeAsn, node, assign(tokens));
    }
    node
}

fn expr(tokens: &mut Tokens) -> Box<Node> {
    let node = assign(tokens);
    node
}

fn stmt(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let node = expr(tokens);
    if tokens.expect_op(";") {
        Ok(node)
    } else {
        Err(format!("Cannot find semicolon!"))
    }
}

pub fn program(tokens: &mut Tokens) -> Result<Vec<Box<Node>>, String> {
    let mut nodes: Vec<Box<Node>> = Vec::new();
    while tokens.has_next() {
        stmt(tokens).map(|node| nodes.push(node))?;
    }
    Ok(nodes)
}
