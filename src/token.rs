use std::str;

use TokenKind::*;

#[derive(PartialEq)]
pub enum TokenKind {
    TokenOp(String),
    TokenNum(u32),
    TokenVar(String),
    TokenEnd,
}

pub struct Token {
    kind: TokenKind,
    pos: usize,
}

pub struct Tokens {
    list: Vec<Token>,
    current: usize,
}

impl Tokens {
    pub fn expect_num(&mut self) -> Option<u32> {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenNum(num) => {
                self.current += 1;
                Some(*num)
            },
            _ => None
        }
    }

    pub fn expect_op(&mut self, expect: &str) -> bool {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenOp(op) => {
                if op == expect {
                    self.current += 1;
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }

    pub fn expect_var(&mut self) -> Option<&str> {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenVar(var) => {
                self.current += 1;
                Some(var.as_str())
            },
            _ => None
        }
    }

    pub fn has_next(&self) -> bool {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenEnd => false,
            _ => true,
        }
    }

    pub fn head(&self) -> usize {
        let cur_tok = &self.list[self.current];
        cur_tok.pos
    }

    pub fn new(v: Vec<Token>) -> Self {
        Self {
            list: v,
            current: 0,
        }
    }
}

pub fn tokenize(formula: &str) -> Result<Vec<Token>, String> {
    let mut v: Vec<Token> = Vec::new();
    let mut num_tmp: Vec<u8> = Vec::new();
    let mut op_tmp: Vec<u8> = Vec::new();
    let mut index = 0;
    let mut pos = 0;
    let bytes = formula.as_bytes();
    let len = bytes.len();

    while index < len {
        match bytes[index] {
            b'0'..=b'9' => {
                num_tmp.push(bytes[index]);
                if pos == 0 {
                    pos = index;
                }
                if (index + 1 < len && 
                    !b"0123456789".contains(&bytes[index + 1])) ||
                   index + 1 == len {
                    let num = str::from_utf8(&num_tmp).unwrap()
                              .parse().expect("Cannot parse!");
                    v.push(Token { kind: TokenNum(num), pos: pos });
                    num_tmp.clear();
                    pos = 0;
                }
            },
            b'+' | b'-' |
            b'*' | b'/' |
            b'(' | b')' |
            b';' => {
                let op = str::from_utf8(&bytes[index].to_be_bytes()).unwrap().to_string();
                v.push(Token { kind: TokenOp(op), pos: index});
            },
            b'<' | b'>' |
            b'=' | b'!' => {
                op_tmp.push(bytes[index]);
                if pos == 0 {
                    pos = index;
                }
                if (index + 1 < len && 
                    !b"<>=!".contains(&bytes[index + 1])) ||
                   index + 1 == len {
                    let op = str::from_utf8(&op_tmp).unwrap().to_string();
                    v.push(Token { kind: TokenOp(op), pos: pos});
                    op_tmp.clear();
                    pos = 0;
                }
            },
            b'a'..=b'z' => {
                let var = str::from_utf8(&bytes[index].to_be_bytes()).unwrap().to_string();
                v.push(Token { kind: TokenVar(var), pos: index});
            },
            b' ' | b'\t'| b'\n' => (),
            _ => return Err(format!("Cannot tokenize!")),
        }
        index += 1;
    }

    v.push(Token { kind: TokenEnd, pos: index });

    Ok(v)
}
