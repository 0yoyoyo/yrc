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
    TokenStr(String),
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

    pub fn expect_bl(&mut self) -> Option<bool> {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenRsv(word) => {
                if word == "true" {
                    self.current += 1;
                    Some(true)
                } else if word == "false" {
                    self.current += 1;
                    Some(false)
                } else {
                    None
                }
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

    pub fn expect_str(&mut self) -> Option<&str> {
        let cur_tok = &self.list[self.current];
        match &cur_tok.kind {
            TokenStr(s) => {
                self.current += 1;
                Some(s.as_str())
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
        self.list.get(self.current - offset).map(|tok| tok.pos)
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

fn lex_cmp(bytes: &[u8], cur: &mut usize) -> Token {
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

fn lex_arw(bytes: &[u8], cur: &mut usize) -> Token {
    let mut tmp: Vec<u8> = Vec::new();
    let pos = *cur;
    loop {
        tmp.push(bytes[*cur]);
        *cur += 1;
        if (*cur >= bytes.len()) ||
           (!b">".contains(&bytes[*cur])) {
            let op = str::from_utf8(&tmp)
                .unwrap()
                .to_string();
            return Token::new(TokenOp(op), pos);
        }
    }
}

fn lex_str(bytes: &[u8], cur: &mut usize) -> Token {
    let mut tmp: Vec<u8> = Vec::new();
    let pos = *cur;
    // Skip first "
    *cur += 1;
    loop {
        tmp.push(bytes[*cur]);
        *cur += 1;
        if (*cur >= bytes.len()) ||
           (bytes[*cur] == b'\"') {
            // Skip end "
            *cur += 1;
            let s = str::from_utf8(&tmp)
                .unwrap()
                .to_string();
            return Token::new(TokenStr(s), pos);
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
               0123456789_"
            .contains(&bytes[*cur])) {
            let name = str::from_utf8(&tmp)
                .unwrap()
                .to_string();
            if name == "fn"       ||
               name == "let"      ||
               name == "static"   ||
               name == "if"       ||
               name == "else"     ||
               name == "for"      ||
               name == "while"    ||
               name == "break"    ||
               name == "continue" ||
               name == "return"   ||
               name == "i8"       ||
               name == "i16"      ||
               name == "i32"      ||
               name == "i64"      ||
               name == "u8"       ||
               name == "u16"      ||
               name == "u32"      ||
               name == "u64"      ||
               name == "bool"     ||
               name == "str"      ||
               name == "true"     ||
               name == "false"    {
                return Token::new(TokenRsv(name), pos);
            } else {
                return Token::new(TokenIdt(name), pos);
            }
        }
    }
}

fn skip_line_comment(bytes: &[u8], cur: &mut usize) {
    // Skip // of line top
    *cur += 2;
    loop {
        if (*cur >= bytes.len()) ||
           (b"\n".contains(&bytes[*cur])) {
            // Skip \n of line end
            *cur += 1;
            break;
        }
        *cur += 1;
    }
}

fn skip_block_comment(bytes: &[u8], cur: &mut usize) {
    // Skip /* of beginning
    *cur += 2;
    loop {
        if ((*cur >= bytes.len()) ||
           (b"*".contains(&bytes[*cur]))) &&
           ((*cur + 1 >= bytes.len()) ||
           (b"/".contains(&bytes[*cur + 1]))) {
            // Skip */ of end
            *cur += 2;
            break;
        }
        *cur += 1;
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
            b'+' | b'*' |
            b'(' | b')' |
            b'[' | b']' |
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
                let token = lex_cmp(bytes, &mut cur);
                tokens.push(token);
            },
            b'-' => {
                let token = lex_arw(bytes, &mut cur);
                tokens.push(token);
            },
            b'\"' => {
                let token = lex_str(bytes, &mut cur);
                tokens.push(token);
            },
            b'A'..=b'Z' |
            b'a'..=b'z' |
            b'_' => {
                let token = lex_word(bytes, &mut cur);
                tokens.push(token);
            },
            b'/' => {
                if (cur < bytes.len()) &&
                   (b"/".contains(&bytes[cur + 1])) {
                    skip_line_comment(bytes, &mut cur);
                } else if (cur < bytes.len()) &&
                          (b"*".contains(&bytes[cur + 1])) {
                    skip_block_comment(bytes, &mut cur);
                } else {
                    let op = str::from_utf8(&bytes[cur].to_ne_bytes())
                        .unwrap()
                        .to_string();
                    tokens.push(Token::new(TokenOp(op), cur));
                    cur += 1;
                }
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
