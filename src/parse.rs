use std::fmt;

use super::token::Tokens;

use BinaryOpKind::*;
use UnaryOpKind::*;
use ParseErrorKind::*;

#[derive(Debug)]
pub enum ParseErrorKind {
    NumberExpected,
    FuncExpected,
    VariableExpected,
    TypeExpected,
    ArgExpected,
    ParenExpected,
    ScolonExpected,
    BlockExpected,
    TypeInvalid,
    UnknownVariable,
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

    fn new_with_offset(e: ParseErrorKind, toks: &Tokens, offset: usize) -> Self {
        ParseError {
            error: e,
            pos: toks.head_before(offset).unwrap_or(0),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}^ ", " ".repeat(self.pos))?;
        match &self.error {
            NumberExpected => write!(f, "Number is expected here!"),
            FuncExpected => write!(f, "Function is expected here!"),
            VariableExpected => write!(f, "Variable is expected here!"),
            TypeExpected => write!(f, "Type is expected here!"),
            ArgExpected => write!(f, "Arguments are needed!"),
            ParenExpected => write!(f, "Parentheses are not closed!"),
            ScolonExpected => write!(f, "Semicolon is needed!"),
            BlockExpected => write!(f, "Block is expected here!"),
            TypeInvalid => write!(f, "Invalid Type!"),
            UnknownVariable => write!(f, "Unknown variable!"),
        }
    }
}

pub const WORDSIZE: usize = 8;

#[derive(Debug, PartialEq)]
pub enum BinaryOpKind {
    BinaryOpAdd,
    BinaryOpSub,
    BinaryOpMul,
    BinaryOpDiv,
    BinaryOpEq,
    BinaryOpNe,
    BinaryOpGr,
    BinaryOpGe,
    BinaryOpAsn,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOpKind {
    UnaryOpRf,
    UnaryOpDrf,
}

#[derive(Debug)]
pub enum Node {
    BinaryOperator {
        kind: BinaryOpKind,
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
        ty: Type,
    },
    GlobalVariable {
        name: String,
        offset: usize,
        ty: Type,
    },
    DeclareGlobal {
        name: String,
        size: usize,
        ty: Type,
    },
    Block {
        nodes: Vec<Box<Node>>,
    },
    Function {
        name: String,
        args: Vec<Box<Node>>,
        stack: usize,
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

fn new_node_bop(kind: BinaryOpKind, lhs: Box<Node>, rhs: Box<Node>) -> Box<Node> {
    let node = Node::BinaryOperator {
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

fn new_node_lvar(offset: usize, ty: Type) -> Box<Node> {
    let node = Node::LocalVariable {
        offset: offset,
        ty: ty,
    };
    Box::new(node)
}

fn new_node_gvar(name: &str, offset: usize, ty: Type) -> Box<Node> {
    let node = Node::GlobalVariable {
        name: name.to_string(),
        offset: offset,
        ty: ty,
    };
    Box::new(node)
}

fn new_node_decg(name: &str, size: usize, ty: Type) -> Box<Node> {
    let node = Node::DeclareGlobal {
        name: name.to_string(),
        size: size,
        ty: ty,
    };
    Box::new(node)
}

fn new_node_blk(nodes: Vec<Box<Node>>) -> Box<Node> {
    let node = Node::Block {
        nodes: nodes,
    };
    Box::new(node)
}

fn new_node_func(name: &str, args: Vec<Box<Node>>, stack: usize, block: Box<Node>) -> Box<Node> {
    let node = Node::Function {
        name: name.to_string(),
        args: args,
        stack: stack,
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

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
    Ary(Box<Type>, usize),
}

struct Lvar {
    name: String,
    ty: Type,
    offset: usize,
}

struct Gvar {
    name: String,
    ty: Type,
}

pub struct Parser {
    lvar_list: Vec<Lvar>,
    gvar_list: Vec<Gvar>,
    block_level: usize,
}

impl Parser {
    fn align_word(n: usize) -> usize {
        n + n % WORDSIZE
    }

    fn type_len(ty: &Type) -> usize {
        match ty {
            Type::Int => 4,
            Type::Ptr(_ty) => WORDSIZE,
            Type::Ary(ty, len) => Self::type_len(&*ty) * len,
        }
    }

    fn get_type(tokens: &mut Tokens) -> Result<Type, ()> {
        if tokens.expect_op("*") {
            Self::get_type(tokens).map(|ty| {
                Type::Ptr(Box::new(ty))
            })
        } else if tokens.expect_op("[") {
            Self::get_type(tokens).and_then(|ty| {
                if tokens.expect_op(";") {
                    tokens.expect_num().ok_or(()).and_then(|num| {
                        if tokens.expect_op("]") {
                            Ok(Type::Ary(Box::new(ty), num as usize))
                        } else {
                            Err(())
                        }
                    })
                } else {
                    Err(())
                }
            })
        } else {
            if tokens.expect_rsv("i32") {
                Ok(Type::Int)
            } else {
                Err(())
            }
        }
    }

    pub fn get_stack_size(&mut self) -> usize {
        let mut total = 0;
        for lvar in &self.lvar_list {
            match &lvar.ty {
                Type::Int => total += WORDSIZE,
                Type::Ptr(_ty) => total += WORDSIZE,
                Type::Ary(ty, len) => { 
                    let size = Self::type_len(&**ty) * len;
                    total += Self::align_word(size);
                },
            }
        }
        total
    }

    fn typ(&mut self, name: &str, tokens: &mut Tokens, is_global: bool) -> Result<Box<Node>, ParseError> {
        if let Ok(ty) = Self::get_type(tokens) {
            if is_global {
                let size = Self::type_len(&ty);
                let new = Gvar {
                    name: name.to_string(),
                    ty: ty.clone(),
                };
                self.gvar_list.push(new);

                Ok(new_node_decg(name, size, ty))
            } else {
                let offset = self.get_stack_size()
                             + Self::align_word(Self::type_len(&ty));
                let new = Lvar {
                    name: name.to_string(),
                    ty: ty.clone(),
                    offset: offset,
                };
                self.lvar_list.push(new);

                Ok(new_node_lvar(offset, ty))
            }
        } else {
            Err(ParseError::new(TypeExpected, tokens))
        }
    }

    fn var(&mut self, name: &str, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let mut i = 0;
        while let Some(lv) = self.lvar_list.get(i) {
            if lv.name == name.to_string() {
                let mut offset = lv.offset;

                if tokens.expect_op("[") {
                    if let Some(num) = tokens.expect_num() {
                        if tokens.expect_op("]") {
                            match &lv.ty {
                                Type::Ary(ty, _) => offset = Self::type_len(&**ty) * (num + 1) as usize,
                                _ => return Err(ParseError::new_with_offset(TypeInvalid, tokens, 4)),
                            }
                        } else {
                            return Err(ParseError::new(ParenExpected, tokens));
                        }
                    } else {
                        return Err(ParseError::new(NumberExpected, tokens));
                    }
                }

                return Ok(new_node_lvar(offset, lv.ty.clone()));
            }
            i += 1;
        }

        i = 0;
        while let Some(gv) = self.gvar_list.get(i) {
            if gv.name == name.to_string() {
                let mut offset = 0;

                if tokens.expect_op("[") {
                    if let Some(num) = tokens.expect_num() {
                        if tokens.expect_op("]") {
                            match &gv.ty {
                                Type::Ary(ty, _) => offset = Self::type_len(&**ty) * num as usize,
                                _ => return Err(ParseError::new_with_offset(TypeInvalid, tokens, 4)),
                            }
                        } else {
                            return Err(ParseError::new(ParenExpected, tokens));
                        }
                    } else {
                        return Err(ParseError::new(NumberExpected, tokens));
                    }
                }

                return Ok(new_node_gvar(name, offset, gv.ty.clone()));
            }
            i += 1;
        }

        Err(ParseError::new_with_offset(UnknownVariable, tokens, 1))
    }

    fn bind(&mut self, tokens: &mut Tokens, is_global: bool) -> Result<Box<Node>, ParseError> {
        if let Some(name) = tokens.expect_idt() {
            // Get ownership
            let name = &name.to_string();

            if tokens.expect_op(":") {
                self.typ(name, tokens, is_global)
            } else {
                Err(ParseError::new(TypeExpected, tokens))
            }
        } else {
            Err(ParseError::new(VariableExpected, tokens))
        }
    }

    fn blk(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        self.block_level += 1;
        let mut nodes: Vec<Box<Node>> = Vec::new();
        while !tokens.expect_op("}") {
            match self.stmt(tokens) {
                Ok(node) => nodes.push(node),
                Err(e) => return Err(e),
            }
        }
        self.block_level -= 1;
        Ok(new_node_blk(nodes))
    }

    fn primary(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        if tokens.expect_op("(") {
            let node = self.expr(tokens)?;
            if tokens.expect_op(")") {
                Ok(node)
            } else {
                Err(ParseError::new(ParenExpected, tokens))
            }
        } else if let Some(name) = tokens.expect_idt() {
            // Get ownership
            let name = &name.to_string();

            if tokens.expect_op("(") {
                let mut args: Vec<Box<Node>> = Vec::new();
                while !tokens.expect_op(")") {
                    let arg = self.expr(tokens)?;
                    args.push(arg);
                    if tokens.expect_op(",") {
                        continue;
                    }
                }

                Ok(new_node_call(name, args))
            } else {
                self.var(name, tokens)
            }
        } else {
            let num = tokens.expect_num()
                .ok_or(ParseError::new(NumberExpected, tokens))?;
            Ok(new_node_num(num))
        }
    }

    fn unary(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        if tokens.expect_op("&") {
            self.unary(tokens)
                .map(|node| new_node_uop(UnaryOpRf, node))
        } else if tokens.expect_op("*") {
            self.unary(tokens)
                .map(|node| new_node_uop(UnaryOpDrf, node))
        } else if tokens.expect_op("-") {
            self.primary(tokens)
                .map(|rhs| new_node_bop(BinaryOpSub, new_node_num(0), rhs))
        } else {
            self.primary(tokens)
        }
    }

    fn mul(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let mut node = self.unary(tokens)?;
        while tokens.has_next() {
            if tokens.expect_op("*") {
                let rhs = self.unary(tokens)?;
                node = new_node_bop(BinaryOpMul, node, rhs);
            } else if tokens.expect_op("/") {
                let rhs = self.unary(tokens)?;
                node = new_node_bop(BinaryOpDiv, node, rhs);
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn add(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let mut node = self.mul(tokens)?;
        while tokens.has_next() {
            if tokens.expect_op("+") {
                let rhs = self.mul(tokens)?;
                node = new_node_bop(BinaryOpAdd, node, rhs);
            } else if tokens.expect_op("-") {
                let rhs = self.mul(tokens)?;
                node = new_node_bop(BinaryOpSub, node, rhs);
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn relational(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let mut node = self.add(tokens)?;
        while tokens.has_next() {
            if tokens.expect_op("<") {
                let rhs = self.add(tokens)?;
                node = new_node_bop(BinaryOpGr, node, rhs);
            } else if tokens.expect_op("<=") {
                let rhs = self.add(tokens)?;
                node = new_node_bop(BinaryOpGe, node, rhs);
            } else if tokens.expect_op(">") {
                let lhs = self.add(tokens)?;
                node = new_node_bop(BinaryOpGr, lhs, node);
            } else if tokens.expect_op(">=") {
                let lhs = self.add(tokens)?;
                node = new_node_bop(BinaryOpGe, lhs, node);
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn equality(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let mut node = self.relational(tokens)?;
        while tokens.has_next() {
            if tokens.expect_op("==") {
                let rhs = self.relational(tokens)?;
                node = new_node_bop(BinaryOpEq, node, rhs);
            } else if tokens.expect_op("!=") {
                let rhs = self.relational(tokens)?;
                node = new_node_bop(BinaryOpNe, node, rhs);
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn assign(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        self.equality(tokens)
            .and_then(|node| {
                if tokens.expect_op("=") {
                    self.assign(tokens)
                        .map(|rhs| new_node_bop(BinaryOpAsn, node, rhs))
                } else {
                    Ok(node)
                }
            })
    }

    fn expr(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        self.assign(tokens)
    }

    fn func(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        if let Some(name) = tokens.expect_idt() {
            // Get ownership
            let name = &name.to_string();

            if !tokens.expect_op("(") {
                return Err(ParseError::new(ArgExpected, tokens));
            }

            let mut args: Vec<Box<Node>> = Vec::new();
            while !tokens.expect_op(")") {
                if let Some(name) = tokens.expect_idt() {
                    // Get ownership
                    let name = &name.to_string();

                    if tokens.expect_op(":") {
                        let node = self.typ(name, tokens, false)?;
                        args.push(node);
                    } else {
                        return Err(ParseError::new(TypeExpected, tokens));
                    }
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
            let block = self.blk(tokens)?;

            let stack = self.get_stack_size();
            self.lvar_list.clear();

            Ok(new_node_func(name, args, stack, block))
        } else {
            Err(ParseError::new(FuncExpected, tokens))
        }
    }

    fn ifel(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let node: Box<Node>;

        let cond = self.expr(tokens)?;

        let ibody: Box<Node>;
        if tokens.expect_op("{") {
            ibody = self.blk(tokens)?;
        } else {
            ibody = self.stmt(tokens)?;
        }

        if tokens.expect_rsv("else") {
            let ebody: Box<Node>;
            if tokens.expect_op("{") {
                ebody = self.blk(tokens)?;
            } else {
                ebody = self.stmt(tokens)?;
            }
            node = new_node_ifel(cond, ibody, ebody);
        } else {
            node = new_node_if(cond, ibody);
        }

        Ok(node)
    }

    fn whl(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let cond = self.expr(tokens)?;

        let body: Box<Node>;
        if tokens.expect_op("{") {
            body = self.blk(tokens)?;
        } else {
            body = self.stmt(tokens)?;
        }

        Ok(new_node_whl(cond, body))
    }

    fn stmt(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let node: Box<Node>;
        if tokens.expect_rsv("fn") {
            node = self.func(tokens)?;
            return Ok(node);
        } else if tokens.expect_rsv("if") {
            node = self.ifel(tokens)?;
            return Ok(node);
        } else if tokens.expect_rsv("while") {
            node = self.whl(tokens)?;
            return Ok(node);
        } else if tokens.expect_rsv("let") {
            node = self.bind(tokens, false)?;
        } else if tokens.expect_rsv("static") {
            node = self.bind(tokens, true)?;
        } else if tokens.expect_rsv("return") {
            let rhs = self.expr(tokens)?;
            node = new_node_ret(rhs);
        } else {
            node = self.expr(tokens)?;
        }

        if tokens.expect_op(";") {
            Ok(node)
        } else {
            Err(ParseError::new(ScolonExpected, tokens))
        }
    }

    pub fn program(&mut self, tokens: &mut Tokens) -> Result<Vec<Box<Node>>, ParseError> {
        let mut nodes: Vec<Box<Node>> = Vec::new();
        while tokens.has_next() {
            match self.stmt(tokens) {
                Ok(node) => nodes.push(node),
                Err(e) => return Err(e),
            }
        }

        #[cfg(feature="trace")]
        println!(" Nodes {:?}", nodes);

        Ok(nodes)
    }

    pub fn new() -> Self {
        Parser {
            lvar_list: Vec::new(),
            gvar_list: Vec::new(),
            block_level: 0,
        }
    }
}
