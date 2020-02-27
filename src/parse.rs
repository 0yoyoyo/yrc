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
    Box::new(node)
}

fn new_node_num(val: u32) -> Box<Node> {
    let node = Node::Number {
        val: val,
    };
    Box::new(node)
}

fn new_node_var(var: &str) -> Box<Node> {
    let node = Node::LocalVariable {
        offset: ((var.as_bytes()[0] - b'a' + 1) * 8) as usize,
    };
    Box::new(node)
}

fn primary(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    if tokens.expect_op("(") {
        let node = expr(tokens)?;
        if tokens.expect_op(")") {
            Ok(node)
        } else {
            Err(format!("Cannot find right parenthesis!"))
        }
    } else if let Some(var) = tokens.expect_var() {
        Ok(new_node_var(var))
    } else {
        let num = tokens.expect_num()
                        .ok_or(format!("{}^ Not a number!",
                            " ".repeat(tokens.head())))?;
        Ok(new_node_num(num))
    }
}

fn unary(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    if tokens.expect_op("-") {
        primary(tokens)
            .map(|rhs| new_node(NodeSub, new_node_num(0), rhs))
    } else {
        primary(tokens)
    }
}

fn mul(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node = unary(tokens)?;
    while tokens.has_next() {
        if tokens.expect_op("*") {
            let rhs = unary(tokens)?;
            node = new_node(NodeMul, node, rhs);
        } else if tokens.expect_op("/") {
            let rhs = unary(tokens)?;
            node = new_node(NodeDiv, node, rhs);
        } else {
            break;
        }
    }
    Ok(node)
}

fn add(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node = mul(tokens)?;
    while tokens.has_next() {
        if tokens.expect_op("+") {
            let rhs = mul(tokens)?;
            node = new_node(NodeAdd, node, rhs);
        } else if tokens.expect_op("-") {
            let rhs = mul(tokens)?;
            node = new_node(NodeSub, node, rhs);
        } else {
            break;
        }
    }
    Ok(node)
}

fn relational(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node = add(tokens)?;
    while tokens.has_next() {
        if tokens.expect_op("<") {
            let rhs = add(tokens)?;
            node = new_node(NodeGr, node, rhs);
        } else if tokens.expect_op("<=") {
            let rhs = add(tokens)?;
            node = new_node(NodeGe, node, rhs);
        } else if tokens.expect_op(">") {
            let lhs = add(tokens)?;
            node = new_node(NodeGr, lhs, node);
        } else if tokens.expect_op(">=") {
            let lhs = add(tokens)?;
            node = new_node(NodeGe, lhs, node);
        } else {
            break;
        }
    }
    Ok(node)
}

fn equality(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    let mut node = relational(tokens)?;
    while tokens.has_next() {
        if tokens.expect_op("==") {
            let rhs = relational(tokens)?;
            node = new_node(NodeEq, node, rhs);
        } else if tokens.expect_op("!=") {
            let rhs = relational(tokens)?;
            node = new_node(NodeNe, node, rhs);
        } else {
            break;
        }
    }
    Ok(node)
}

fn assign(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    equality(tokens)
        .and_then(|node| {
            if tokens.expect_op("=") {
                assign(tokens)
                    .map(|rhs| new_node(NodeAsn, node, rhs))
            } else {
                Ok(node)
            }
        })
}

fn expr(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    assign(tokens)
}

fn stmt(tokens: &mut Tokens) -> Result<Box<Node>, String> {
    expr(tokens)
        .and_then(|node| {
            if tokens.expect_op(";") {
                Ok(node)
            } else {
                Err(format!("Cannot find semicolon!"))
            }
        })
}

pub fn program(tokens: &mut Tokens) -> Result<Vec<Box<Node>>, String> {
    let mut nodes: Vec<Box<Node>> = Vec::new();
    while tokens.has_next() {
        match stmt(tokens) {
            Ok(node) => nodes.push(node),
            Err(e) => return Err(e),
        }
    }
    Ok(nodes)
}
