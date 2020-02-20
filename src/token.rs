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

impl Token {
    fn get_num(&self) -> std::option::Option<u32> {
        match self.kind {
            TokenNum(num) => Some(num),
            _ => None,
        }
    }

    fn get_op(&self) -> std::option::Option<&str> {
        match &self.kind {
            TokenOp(op) => Some(&op),
            _ => None,
        }
    }

    fn get_pos(&self) -> usize {
        self.pos
    }
}

pub struct Tokens {
    list: Vec<Token>,
    current: usize,
}

impl Tokens {
    pub fn expect_num(&mut self) -> u32 {
        if let Some(num) = self.list[self.current].get_num() {
            self.current += 1;
            num
        } else {
            print!("{}^ ", " ".repeat(self.list[self.current].get_pos()));
            println!("Not a number!");
            std::process::exit(1);
        }
    }

    pub fn expect_op(&mut self, expect: &str) -> bool {
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

    pub fn has_next(&self) -> bool {
        match self.list[self.current].kind {
            TokenEnd => false,
            _ => true,
        }
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
            b'(' | b')' => {
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
