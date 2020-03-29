use std::str;
use std::fmt;

use TokenKind::*;
use TokenErrorKind::*;

#[derive(Debug)]
pub enum TokenErrorKind {
    CannotTokenize,
}

#[derive(Debug)]
pub struct TokenError {
    error: TokenErrorKind,
    pos: usize,
}

impl TokenError {
    fn new(e: TokenErrorKind, p: usize) -> Self {
        TokenError {
            error: e,
            pos: p,
        }
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}^ ", " ".repeat(self.pos))?;
        match &self.error {
            CannotTokenize => write!(f, "Cannot tokenize!"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    TokenOp(String),
    TokenNum(u32),
    TokenIdt(String),
    TokenRsv(String),
    TokenEnd,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    pos: usize,
}

impl Token {
    fn new(k: TokenKind, p: usize) -> Self {
        Token {
            kind: k,
            pos: p,
        }
    }
}

#[derive(Debug)]
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
            TokenOp(op) if op == expect => {
                self.current += 1;
                true
            },
            _ => false
        }
    }

    pub fn expect_idt(&mut self) -> Option<&str> {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenIdt(name) => {
                self.current += 1;
                Some(name.as_str())
            },
            _ => None
        }
    }

    pub fn expect_rsv(&mut self, expect: &str) -> bool {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenRsv(word) if word == expect => {
                self.current += 1;
                true
            },
            _ => false
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

    pub fn head_before(&self, offset: usize) -> Option<usize> {
        self.list.get(self.current - offset).and_then(|tok| Some(tok.pos))
    }

    pub fn new(v: Vec<Token>) -> Self {
        Self {
            list: v,
            current: 0,
        }
    }
}

fn lex_num(bytes: &[u8], cur: &mut usize) -> Token {
    let mut tmp: Vec<u8> = Vec::new();
    let pos = *cur;
    loop {
        tmp.push(bytes[*cur]);
        *cur += 1;
        if (*cur >= bytes.len()) ||
           (!b"0123456789".contains(&bytes[*cur])) {
            let num = str::from_utf8(&tmp)
                .unwrap()
                .parse()
                .unwrap();
            return Token::new(TokenNum(num), pos);
        }
    }
}

fn lex_op(bytes: &[u8], cur: &mut usize) -> Token {
    let mut tmp: Vec<u8> = Vec::new();
    let pos = *cur;
    loop {
        tmp.push(bytes[*cur]);
        *cur += 1;
        if (*cur >= bytes.len()) ||
           (!b"<>=!".contains(&bytes[*cur])) {
            let op = str::from_utf8(&tmp)
                .unwrap()
                .to_string();
            return Token::new(TokenOp(op), pos);
        }
    }
}

fn lex_word(bytes: &[u8], cur: &mut usize) -> Token {
    let mut tmp: Vec<u8> = Vec::new();
    let pos = *cur;
    loop {
        tmp.push(bytes[*cur]);
        *cur += 1;
        if (*cur >= bytes.len()) ||
           (!b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
               abcdefghijklmnopqrstuvwxyz\
               0123456789"
            .contains(&bytes[*cur])) {
            let name = str::from_utf8(&tmp)
                .unwrap()
                .to_string();
            if name == "fn"      .to_string() ||
               name == "let"     .to_string() ||
               name == "if"      .to_string() ||
               name == "else"    .to_string() ||
               name == "for"     .to_string() ||
               name == "while"   .to_string() ||
               name == "break"   .to_string() ||
               name == "continue".to_string() ||
               name == "return"  .to_string() {
                return Token::new(TokenRsv(name), pos);
            } else if name == "i32".to_string() {
                return Token::new(TokenRsv(name), pos);
            } else {
                return Token::new(TokenIdt(name), pos);
            }
        }
    }
}

pub fn tokenize(formula: &str) -> Result<Vec<Token>, TokenError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut cur = 0;
    let bytes = formula.as_bytes();

    while cur < bytes.len() {
        match bytes[cur] {
            b'0'..=b'9' => {
                let token = lex_num(bytes, &mut cur);
                tokens.push(token);
            },
            b'+' | b'-' |
            b'*' | b'/' |
            b'(' | b')' |
            b'{' | b'}' |
            b'&' | b',' |
            b':' | b';' => {
                let op = str::from_utf8(&bytes[cur].to_ne_bytes())
                    .unwrap()
                    .to_string();
                tokens.push(Token::new(TokenOp(op), cur));
                cur += 1;
            },
            b'<' | b'>' |
            b'=' | b'!' => {
                let token = lex_op(bytes, &mut cur);
                tokens.push(token);
            },
            b'A'..=b'Z' |
            b'a'..=b'z' => {
                let token = lex_word(bytes, &mut cur);
                tokens.push(token);
            },
            b' ' | b'\t'| b'\n' => cur += 1,
            _ => return Err(TokenError::new(CannotTokenize, cur)),
        }
    }

    tokens.push(Token::new(TokenEnd, cur));

    #[cfg(feature="trace")]
    println!(" Tokens {:?}", tokens);

    Ok(tokens)
}
