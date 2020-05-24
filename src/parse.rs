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
    ColonExpected,
    BlockExpected,
    TypeInvalid,
    UnknownVariable,
    NotInTop,
    ExprInvalid,
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
            ColonExpected => write!(f, "Colon is needed!"),
            BlockExpected => write!(f, "Block is expected here!"),
            TypeInvalid => write!(f, "Invalid Type!"),
            UnknownVariable => write!(f, "Unknown variable!"),
            NotInTop => write!(f, "Cannot use in top level"),
            ExprInvalid => write!(f, "Invalid expression!"),
        }
    }
}

const WORDSIZE: usize = 8;

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
        rhs: Box<Node>,
    },
    Number {
        val: u32,
    },
    Bool {
        bl: bool,
    },
    StrLiteral {
        s: String,
        label: usize,
    },
    LocalVariable {
        offset: usize,
        ty: Type,
    },
    DeclareLocal {
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
        ty: Type,
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
        ty: Type,
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

fn new_node_uop(kind: UnaryOpKind, rhs: Box<Node>) -> Box<Node> {
    let node = Node::UnaryOperator {
        kind: kind,
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

fn new_node_bl(bl: bool) -> Box<Node> {
    let node = Node::Bool {
        bl: bl,
    };
    Box::new(node)
}

fn new_node_str(s: &str, label: usize) -> Box<Node> {
    let node = Node::StrLiteral {
        s: s.to_string(),
        label: label,
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

fn new_node_decl(offset: usize, ty: Type) -> Box<Node> {
    let node = Node::DeclareLocal {
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

fn new_node_call(name: &str, args: Vec<Box<Node>>, ty: Type) -> Box<Node> {
    let node = Node::Call {
        name: name.to_string(),
        args: args,
        ty: ty,
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

fn new_node_ret(rhs: Box<Node>, ty: Type) -> Box<Node> {
    let node = Node::Return {
        rhs: rhs,
        ty: ty,
    };
    Box::new(node)
}

fn align_double_word(n: usize) -> usize {
    let dw = WORDSIZE * 2;
    if n % dw != 0 {
        n + (dw - n % dw)
    } else {
        n
    }
}

pub fn type_size(ty: &Type) -> usize {
    match ty {
        Type::Int8 => 1,
        Type::Int16 => 2,
        Type::Int32 => 4,
        Type::Int64 => 8,
        Type::Bool => 1,
        Type::Str => unreachable!(), // Str is not first-class type.
        Type::Ptr(_ty) => WORDSIZE,
        Type::Slc(_ty) => WORDSIZE * 2,
        Type::Ary(ty, len) => type_size(&*ty) * len,
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Int8,
    Int16,
    Int32,
    Int64,
    Bool,
    Str,
    Ptr(Box<Type>),
    Slc(Box<Type>),
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

struct Func {
    name: String,
    ty: Type,
}

struct VarInfo {
    name: String,
    ty: Type,
}

pub struct Parser {
    lvar_list: Vec<Lvar>,
    gvar_list: Vec<Gvar>,
    literal_list: Vec<String>,
    func_list: Vec<Func>,
    block_level: usize,
    cur_type: Type,
}

// Production rules
//
// <idt>  ::= IDENTIFIER
// <slit> ::= STRING LITERAL
// <num>  ::= NUMBER
//
// <typ>  ::= TYPE ("i32", "&str", "[i8; 4]", etc.)
//
// <fn_args> ::= (<bind> ("," <bind>)*)?
// <cl_args> ::= (<expr> ("," <expr>)*)?
//
// <sym>  ::= <idt> ("(" <cl_args> ")")?
// <prim> ::= <num> | <slit> | <sym> | "(" <expr> ")"
// <una>  ::= "-"? <prim> | "&" <una> | "*" <una>
// <mul>  ::= <una> ("*" <una> | "/" <una>)*
// <add>  ::= <mul> ("+" <mul> | "-" <mul>)*
// <rel>  ::= <add> ("<" <add> | "<=" <add> | ">" <add> | ">=" <add>)*
// <eql>  ::= <rel> ("==" <rel> | "!=" <rel>)*
// <asn>  ::= <eql> ("=" <asn>)?
//
// <expr> ::= <asn>
// <whl>  ::= "while" <expr> <blk>
// <ifel> ::= "if" <expr> <blk> ("else" <blk>)?
// <ret>  ::= "return" <expr>
// <locl> ::= "let" <bind>
//
// <stmt> ::= <expr> ";" | <locl> ";" | <ret> ";" | <ifel> | <whl>
// <blk>  ::= "{" <stmt>* "}"
// <func> ::= "fn" <idt> "(" <fn_args> ")" "->" <typ> <blk>
// <bind> ::= <idt> ":" <typ>
// <glbl> ::= "static" <bind>
// <top>  ::= <func> | <glbl> ";"
// <pgrm> ::= <top>*
impl Parser {
    pub fn literals(&self) -> &Vec<String> {
        &self.literal_list
    }

    fn stack_size(&mut self) -> usize {
        self.lvar_list.iter()
            .fold(0, |total, lvar| total + type_size(&lvar.ty))
    }

    fn consume_semicolon(&self, tokens: &mut Tokens) -> Result<(), ParseError> {
        if tokens.expect_op(";") {
            Ok(())
        } else {
            Err(ParseError::new(ScolonExpected, tokens))
        }
    }

    fn consume_colon(&self, tokens: &mut Tokens) -> Result<(), ParseError> {
        if tokens.expect_op(":") {
            Ok(())
        } else {
            Err(ParseError::new(ColonExpected, tokens))
        }
    }

    fn func_type(&mut self, name: &str, _tokens: &mut Tokens) -> Result<Type, ParseError> {
        let mut func_iter = self.func_list.iter();
        while let Some(f) = func_iter.next() {
            if f.name != name.to_string() {
                continue;
            }
            return Ok(f.ty.clone());
        }
        Ok(Type::Int8)
        // TODO: Function declaration is needed?
        //Err(ParseError::new_with_offset(UnknownVariable, tokens, 4))
    }

    fn var(&mut self, name: &str, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let mut lvar_iter = self.lvar_list.iter();
        while let Some(lv) = lvar_iter.next() {
            if lv.name != name.to_string() {
                continue;
            }

            if tokens.expect_op("[") {
                let num = tokens.expect_num()
                    .ok_or(ParseError::new(NumberExpected, tokens))?;
                if !tokens.expect_op("]") {
                    return Err(ParseError::new(ParenExpected, tokens));
                }

                if let Type::Ary(ty, _) = &lv.ty {
                    let offset = lv.offset - type_size(&**ty) * num as usize;
                    return Ok(new_node_lvar(offset, *ty.clone()));
                } else {
                    return Err(ParseError::new_with_offset(TypeInvalid, tokens, 4));
                }
            } else {
                return Ok(new_node_lvar(lv.offset, lv.ty.clone()));
            }
        }

        let mut gvar_iter = self.gvar_list.iter();
        while let Some(gv) = gvar_iter.next() {
            if gv.name != name.to_string() {
                continue;
            }

            if tokens.expect_op("[") {
                let num = tokens.expect_num()
                    .ok_or(ParseError::new(NumberExpected, tokens))?;
                if !tokens.expect_op("]") {
                    return Err(ParseError::new(ParenExpected, tokens));
                }

                if let Type::Ary(ty, _) = &gv.ty {
                    let offset = type_size(&**ty) * num as usize;
                    return Ok(new_node_gvar(name, offset, *ty.clone()));
                } else {
                    return Err(ParseError::new_with_offset(TypeInvalid, tokens, 4));
                }
            } else {
                return Ok(new_node_gvar(name, 0, gv.ty.clone()));
            }
        }

        Err(ParseError::new_with_offset(UnknownVariable, tokens, 1))
    }

    fn bind(&mut self, tokens: &mut Tokens) -> Result<VarInfo, ParseError> {
        let name = tokens.expect_idt()
            .map(|s| s.to_string()) // Get ownership
            .ok_or(ParseError::new(VariableExpected, tokens))?;

        self.consume_colon(tokens)?;
        let ty = self.typ(tokens)?;

        Ok(VarInfo { name: name, ty: ty })
    }

    fn typ(&self, tokens: &mut Tokens) -> Result<Type, ParseError> {
        if tokens.expect_op("&") {
            let ty = self.typ(tokens)?;
            match ty {
                Type::Int8 | Type::Int16 |
                Type::Int32 | Type::Int64 |
                Type::Bool |
                Type::Ptr(_) | Type::Slc(_) => {
                    Ok(Type::Ptr(Box::new(ty)))
                },
                Type::Str | Type::Ary(_, _) => {
                    Ok(Type::Slc(Box::new(ty)))
                },
            }
        } else if tokens.expect_op("[") {
            let ty = self.typ(tokens)?;
            self.consume_semicolon(tokens)?;
            let num = tokens.expect_num()
                .ok_or(ParseError::new(NumberExpected, tokens))?;

            if !tokens.expect_op("]") {
                return Err(ParseError::new(ParenExpected, tokens));
            }

            Ok(Type::Ary(Box::new(ty), num as usize))
        } else {
            if tokens.expect_rsv("i8") {
                Ok(Type::Int8)
            } else if tokens.expect_rsv("i16") {
                Ok(Type::Int16)
            } else if tokens.expect_rsv("i32") {
                Ok(Type::Int32)
            } else if tokens.expect_rsv("i64") {
                Ok(Type::Int64)
            } else if tokens.expect_rsv("bool") {
                Ok(Type::Bool)
            } else if tokens.expect_rsv("str") {
                Ok(Type::Str)
            } else {
                Err(ParseError::new(TypeExpected, tokens))
            }
        }
    }

    fn primary(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        if let Some(num) = tokens.expect_num() {
            Ok(new_node_num(num))
        } else if let Some(bl) = tokens.expect_bl() {
            Ok(new_node_bl(bl))
        } else if let Some(slit) = tokens.expect_str() {
            self.literal_list.push(slit.to_string());
            Ok(new_node_str(slit, self.literal_list.len() - 1))
        } else if let Some(name) = tokens.expect_idt() {
            let name = name.to_string(); // Get ownership
            if tokens.expect_op("(") {
                let mut args: Vec<Box<Node>> = Vec::new();
                while !tokens.expect_op(")") {
                    let arg = self.expr(tokens)?;
                    args.push(arg);
                    if tokens.expect_op(",") {
                        continue;
                    }
                }
                let ty = self.func_type(&name, tokens)?;
                Ok(new_node_call(&name, args, ty))
            } else {
                self.var(&name, tokens)
            }
        } else if tokens.expect_op("(") {
            let node = self.expr(tokens)?;
            if !tokens.expect_op(")") {
                return Err(ParseError::new(ParenExpected, tokens))
            }
            Ok(node)
        } else {
            Err(ParseError::new(ExprInvalid, tokens))
        }
    }

    fn unary(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        if tokens.expect_op("&") {
            self.unary(tokens)
                .map(|rhs| new_node_uop(UnaryOpRf, rhs))
        } else if tokens.expect_op("*") {
            self.unary(tokens)
                .map(|rhs| new_node_uop(UnaryOpDrf, rhs))
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
        let node = self.equality(tokens)?;

        if tokens.expect_op("=") {
            self.assign(tokens)
                .map(|rhs| new_node_bop(BinaryOpAsn, node, rhs))
        } else {
            Ok(node)
        }
    }

    fn expr(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        self.assign(tokens)
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

    fn func(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let name = tokens.expect_idt()
            .map(|s| s.to_string())
            .ok_or(ParseError::new(FuncExpected, tokens))?;

        if !tokens.expect_op("(") {
            return Err(ParseError::new(ArgExpected, tokens));
        }

        let mut args: Vec<Box<Node>> = Vec::new();
        while !tokens.expect_op(")") {
            let vi = self.bind(tokens)?;

            let offset = self.stack_size() + type_size(&vi.ty);
            let new = Lvar {
                name: vi.name.clone(),
                ty: vi.ty,
                offset: offset,
            };
            self.lvar_list.push(new);

            let node = self.var(&vi.name, tokens)?;
            args.push(node);
            if tokens.expect_op(",") {
                continue;
            }
        }

        self.cur_type = if tokens.expect_op("->") {
            self.typ(tokens)?
        } else {
            Type::Int8
        };

        let new = Func {
            name: name.clone(),
            ty: self.cur_type.clone(),
        };
        self.func_list.push(new);

        if !tokens.expect_op("{") {
            return Err(ParseError::new(BlockExpected, tokens));
        }

        let block = self.blk(tokens)?;

        let stack = align_double_word(self.stack_size());
        self.lvar_list.clear();

        Ok(new_node_func(&name, args, stack, block))
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

    fn locl(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let vi = self.bind(tokens)?;

        let offset = self.stack_size() + type_size(&vi.ty);
        let new = Lvar {
            name: vi.name,
            ty: vi.ty.clone(),
            offset: offset,
        };
        self.lvar_list.push(new);

        Ok(new_node_decl(offset, vi.ty))
    }

    fn stmt(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let node: Box<Node>;

        if tokens.expect_rsv("if") {
            node = self.ifel(tokens)?;
        } else if tokens.expect_rsv("while") {
            node = self.whl(tokens)?;
        } else if tokens.expect_rsv("let") {
            node = self.locl(tokens)?;
            self.consume_semicolon(tokens)?;
        } else if tokens.expect_rsv("return") {
            let rhs = self.expr(tokens)?;
            node = new_node_ret(rhs, self.cur_type.clone());
            self.consume_semicolon(tokens)?;
        } else {
            node = self.expr(tokens)?;
            self.consume_semicolon(tokens)?;
        }

        Ok(node)
    }

    fn glbl(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        let vi = self.bind(tokens)?;

        let size = type_size(&vi.ty);
        let new = Gvar {
            name: vi.name.clone(),
            ty: vi.ty.clone(),
        };
        self.gvar_list.push(new);

        Ok(new_node_decg(&vi.name, size, vi.ty))
    }

    fn top(&mut self, tokens: &mut Tokens) -> Result<Box<Node>, ParseError> {
        if tokens.expect_rsv("fn") {
            self.func(tokens)
        } else if tokens.expect_rsv("static") {
            let node = self.glbl(tokens)?;
            self.consume_semicolon(tokens)?;
            Ok(node)
        } else {
            Err(ParseError::new(NotInTop, tokens))
        }
    }

    pub fn program(&mut self, tokens: &mut Tokens) -> Result<Vec<Box<Node>>, ParseError> {
        let mut nodes: Vec<Box<Node>> = Vec::new();
        while tokens.has_next() {
            match self.top(tokens) {
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
            literal_list: Vec::new(),
            func_list: Vec::new(),
            block_level: 0,
            cur_type: Type::Int8,
        }
    }
}
