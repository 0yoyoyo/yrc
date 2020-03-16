use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use super::token::Tokens;

use NodeKind::*;
use UnaryOpKind::*;
use ParseErrorKind::*;

#[derive(Debug)]
pub enum ParseErrorKind {
    NumberExpected,
    FuncExpected,
    ArgExpected,
    ParenExpected,
    ScolonExpected,
    BlockExpected,
}

#[derive(Debug)]
pub struct ParseError {
    error: ParseErrorKind,
    pos: usize,
}

impl ParseError {
    fn new(e: ParseErrorKind, toks: &Tokens) -> Self {
        ParseError {
            error: e,
            pos: toks.head(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}^ ", " ".repeat(self.pos))?;
        match &self.error {
            NumberExpected => write!(f, "Number is expected here!"),
            FuncExpected => write!(f, "Function is expected here!"),
            ArgExpected => write!(f, "Arguments are needed!"),
            ParenExpected => write!(f, "Parentheses are not closed!"),
            ScolonExpected => write!(f, "Semicolon is needed!"),
            BlockExpected => write!(f, "Block is expected here!"),
        }
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum UnaryOpKind {
    UnaryOpRf,
    UnaryOpDrf,
}

#[derive(Debug)]
pub enum Node {
    Operator {
        kind: NodeKind,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    UnaryOperator {
        kind: UnaryOpKind,
        node: Box<Node>,
    },
    Number {
        val: u32,
    },
    LocalVariable {
        offset: usize,
    },
    Block {
        nodes: Vec<Box<Node>>,
    },
    Function {
        name: String,
        args: Vec<Box<Node>>,
        block: Box<Node>,
    },
    Call {
        name: String,
        args: Vec<Box<Node>>,
    },
    If {
        cond: Box<Node>,
        ibody: Box<Node>,
    },
    IfElse {
        cond: Box<Node>,
        ibody: Box<Node>,
        ebody: Box<Node>,
    },
    While {
        cond: Box<Node>,
        body: Box<Node>,
    },
    Return {
        rhs: Box<Node>,
    },
}

struct Lvar {
    name: String,
    offset: usize,
}

thread_local!(
    static LVAR_LIST: Rc<RefCell<Vec<Lvar>>> = {
        let v: Vec<Lvar> = Vec::new();
        Rc::new(RefCell::new(v))
    }
);

pub fn get_lvar_num() -> usize {
    let list = LVAR_LIST.with(|v| v.clone());
    let num = list.borrow().len();
    num
}

fn new_node(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Box<Node> {
    let node = Node::Operator {
        kind: kind,
        lhs: lhs,
        rhs: rhs,
    };
    Box::new(node)
}

fn new_node_uop(kind: UnaryOpKind, node: Box<Node>) -> Box<Node> {
    let node = Node::UnaryOperator {
        kind: kind,
        node: node,
    };
    Box::new(node)
}

fn new_node_num(val: u32) -> Box<Node> {
    let node = Node::Number {
        val: val,
    };
    Box::new(node)
}

fn new_node_var(name: &str) -> Box<Node> {
    let list = LVAR_LIST.with(|v| v.clone());
    // Set default offset as inputting new variable.
    let mut offset = 8 * (list.borrow().len() + 1);
    let mut i = 0;

    while let Some(lv) = list.borrow().get(i) {
        if lv.name == name.to_string() {
            offset = lv.offset;
            break;
        }
        i += 1;
    }
    if i == list.borrow().len(){
        let new = Lvar {
            name: name.to_string(),
            offset: 8 * (list.borrow().len() + 1),
        };
        list.borrow_mut().push(new);
    }

    let node = Node::LocalVariable {
        offset: offset,
    };
    Box::new(node)
}

fn new_node_block(nodes: Vec<Box<Node>>) -> Box<Node> {
    let node = Node::Block {
        nodes: nodes,
    };
    Box::new(node)
}

fn new_node_func(name: &str, args: Vec<Box<Node>>, block: Box<Node>) -> Box<Node> {
    let node = Node::Function {
        name: name.to_string(),
        args: args,
        block: block,
    };
    Box::new(node)
}

fn new_node_call(name: &str, args: Vec<Box<Node>>) -> Box<Node> {
    let node = Node::Call {
        name: name.to_string(),
        args: args,
    };
    Box::new(node)
}

fn new_node_if(cond: Box<Node>, ibody: Box<Node>) -> Box<Node> {
    let node = Node::If {
        cond: cond,
        ibody: ibody,
    };
    Box::new(node)
}

fn new_node_ifel(cond: Box<Node>, ibody: Box<Node>, ebody: Box<Node>) -> Box<Node> {
    let node = Node::IfElse {
        cond: cond,
        ibody: ibody,
        ebody: ebody,
    };
    Box::new(node)
}

fn new_node_whl(cond: Box<Node>, body: Box<Node>) -> Box<Node> {
    let node = Node::While {
        cond: cond,
        body: body,
    };
    Box::new(node)
}

fn new_node_ret(rhs: Box<Node>) -> Box<Node> {
    let node = Node::Return {
        rhs: rhs,
    };
    Box::new(node)
}

fn block(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    let mut nodes: Vec<Box<Node>> = Vec::new();
    while !tokens.expect_op("}") {
        match stmt(tokens) {
            Ok(node) => nodes.push(node),
            Err(e) => return Err(e),
        }
    }
    Ok(new_node_block(nodes))
}

fn primary(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    if tokens.expect_op("(") {
        let node = expr(tokens)?;
        if tokens.expect_op(")") {
            Ok(node)
        } else {
            Err(ParseError::new(ParenExpected, tokens))
        }
    } else if let Some(name) = tokens.expect_idt() {
        // It looks a little bad...
        let name = &name.to_string().clone();

        if tokens.expect_op("(") {
            let mut args: Vec<Box<Node>> = Vec::new();
            while !tokens.expect_op(")") {
                let arg = expr(tokens)?;
                args.push(arg);
                if tokens.expect_op(",") {
                    continue;
                }
            }

            Ok(new_node_call(name, args))
        } else {
            Ok(new_node_var(name))
        }
    } else {
        let num = tokens.expect_num()
            .ok_or(ParseError::new(NumberExpected, tokens))?;
        Ok(new_node_num(num))
    }
}

fn unary(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    if tokens.expect_op("&") {
        unary(tokens)
            .map(|node| new_node_uop(UnaryOpRf, node))
    } else if tokens.expect_op("*") {
        unary(tokens)
            .map(|node| new_node_uop(UnaryOpDrf, node))
    } else if tokens.expect_op("-") {
        primary(tokens)
            .map(|rhs| new_node(NodeSub, new_node_num(0), rhs))
    } else {
        primary(tokens)
    }
}

fn mul(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
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

fn add(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
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

fn relational(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
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

fn equality(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
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

fn assign(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
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

fn expr(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    assign(tokens)
}

fn func(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    if let Some(name) = tokens.expect_idt() {
        // It looks a little bad...
        let name = &name.to_string().clone();

        if !tokens.expect_op("(") {
            return Err(ParseError::new(ArgExpected, tokens));
        }

        let mut args: Vec<Box<Node>> = Vec::new();
        while !tokens.expect_op(")") {
            if let Some(name) = tokens.expect_idt() {
                args.push(new_node_var(name));
            } else {
                return Err(ParseError::new(ParenExpected, tokens));
            }
            if tokens.expect_op(",") {
                continue;
            }
        }

        if !tokens.expect_op("{") {
            return Err(ParseError::new(BlockExpected, tokens));
        }
        let block = block(tokens)?;

        Ok(new_node_func(name, args, block))
    } else {
        Err(ParseError::new(FuncExpected, tokens))
    }
}

fn ifel(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    let node: Box<Node>;

    let cond = expr(tokens)?;

    let ibody: Box<Node>;
    if tokens.expect_op("{") {
        ibody = block(tokens)?;
    } else {
        ibody = stmt(tokens)?;
    }

    if tokens.expect_rsv("else") {
        let ebody: Box<Node>;
        if tokens.expect_op("{") {
            ebody = block(tokens)?;
        } else {
            ebody = stmt(tokens)?;
        }
        node = new_node_ifel(cond, ibody, ebody);
    } else {
        node = new_node_if(cond, ibody);
    }

    Ok(node)
}

fn whl(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    let cond = expr(tokens)?;

    let body: Box<Node>;
    if tokens.expect_op("{") {
        body = block(tokens)?;
    } else {
        body = stmt(tokens)?;
    }

    Ok(new_node_whl(cond, body))
}

fn stmt(tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
    let node: Box<Node>;
    if tokens.expect_rsv("fn") {
        node = func(tokens)?;
        return Ok(node);
    } else if tokens.expect_rsv("if") {
        node = ifel(tokens)?;
        return Ok(node);
    } else if tokens.expect_rsv("while") {
        node = whl(tokens)?;
        return Ok(node);
    } else if tokens.expect_rsv("return") {
        let rhs = expr(tokens)?;
        node = new_node_ret(rhs);
    } else {
        node = expr(tokens)?;
    }

    if tokens.expect_op(";") {
        Ok(node)
    } else {
        Err(ParseError::new(ScolonExpected, tokens))
    }
}

pub fn program(tokens: &mut Tokens) -> Result<Vec<Box<Node>>, ParseError> {
    let mut nodes: Vec<Box<Node>> = Vec::new();
    while tokens.has_next() {
        match stmt(tokens) {
            Ok(node) => nodes.push(node),
            Err(e) => return Err(e),
        }
    }

    #[cfg(feature="trace")]
    println!(" Nodes {:?}", nodes);

    Ok(nodes)
}
