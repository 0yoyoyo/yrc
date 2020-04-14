use std::fmt;
use std::io;
use std::fs::File;
use std::io::prelude::*;

use super::parse::Node;
use super::parse::BinaryOpKind::*;
use super::parse::UnaryOpKind::*;
use super::parse::Type;
use super::parse::WORDSIZE;

use AsmError::*;

const ARG_REGS_64: [&str; 6] = ["rdi", "rsi", "rdx", "rcx",  "r8",  "r9"];
const ARG_REGS_32: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
const ARG_REGS_16: [&str; 6] = [ "di",  "si",  "dx",  "cx", "r8d", "r9d"];
const ARG_REGS_8:  [&str; 6] = ["dil", "sil",  "dl",  "cl", "r8d", "r9d"];

#[derive(Debug)]
pub enum AsmError {
    Io(io::Error),
    Context,
    DrfErr,
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Io(e) => write!(f, "IO error! ({})", e),
            Context => write!(f, "Lvalue is not assignable!"),
            DrfErr => write!(f, "Dereference target in not pointer type!"),
        }
    }
}

impl From<io::Error> for AsmError {
    fn from(e: io::Error) -> Self {
        AsmError::Io(e)
    }
}

fn type_len(ty: &Type) -> usize {
    match ty {
        Type::Int8 => 1,
        Type::Int16 => 2,
        Type::Int32 => 4,
        Type::Int64 => 8,
        Type::Ptr(_ty) => WORDSIZE,
        Type::Ary(ty, _len) => type_len(&*ty),
    }
}

fn lval_type(node: &Box<Node>) -> Result<&Type, AsmError> {
    match &**node {
        Node::LocalVariable { offset: _, ty } => Ok(ty),
        Node::GlobalVariable { name: _, offset: _, ty } => Ok(ty),
        Node::UnaryOperator { kind, rhs } => {
            match kind {
                UnaryOpDrf => {
                    if let Ok(Type::Ptr(ty)) = lval_type(rhs) {
                        Ok(ty)
                    } else {
                        Err(DrfErr)
                    }
                }
                _ => Err(Context),
            }
        },
        _ => Err(Context),
    }
}

fn lval_size(node: &Box<Node>) -> Result<usize, AsmError> {
    let ty = lval_type(node)?;
    Ok(type_len(ty))
}

pub struct AsmGenerator {
    label_count: usize,
}

impl AsmGenerator {
    fn gen_asm_block(&mut self, f: &mut File, nodes: &Vec<Box<Node>>) -> Result<(), AsmError> {
        let mut iter = nodes.into_iter();
        while let Some(node) = iter.next() {
            self.gen_asm_node(f, node)?;
        }
        Ok(())
    }

    fn gen_asm_lval(&mut self, f: &mut File, node: &Box<Node>) -> Result<(), AsmError> {
        match &**node {
            Node::LocalVariable { offset, ty: _ } => {
                write!(f, "    mov rax, rbp\n")?;
                write!(f, "    sub rax, {}\n", offset)?;
                write!(f, "    push rax\n")?;
                Ok(())
            },
            Node::GlobalVariable { name, offset, ty: _ } => {
                write!(f, "    mov rax, OFFSET FLAT:{}+{}\n", name, offset)?;
                write!(f, "    push rax\n")?;
                Ok(())
            },
            Node::UnaryOperator { kind, rhs } => {
                match kind {
                    UnaryOpDrf => {
                        self.gen_asm_node(f, rhs)?;
                        Ok(())
                    }
                    _ => Err(Context),
                }
            },
            _ => Err(Context),
        }
    }

    fn gen_asm_node(&mut self, f: &mut File, node: &Box<Node>) -> Result<(), AsmError> {
        match &**node {
            Node::Number { val } => {
                write!(f, "    push {}\n", val)?;
            },
            Node::BinaryOperator { kind, lhs, rhs } => {
                if *kind == BinaryOpAsn {
                    self.gen_asm_lval(f, lhs)?;
                } else {
                    self.gen_asm_node(f, lhs)?;
                }
                self.gen_asm_node(f, rhs)?;
                write!(f, "    pop rdi\n")?;
                write!(f, "    pop rax\n")?;
                match kind {
                    BinaryOpAdd => {
                        write!(f, "    add rax, rdi\n")?;
                    },
                    BinaryOpSub => {
                        write!(f, "    sub rax, rdi\n")?;
                    },
                    BinaryOpMul => {
                        write!(f, "    imul rax, rdi\n")?;
                    },
                    BinaryOpDiv => {
                        write!(f, "    cqo\n")?;
                        write!(f, "    idiv rdi\n")?;
                    },
                    BinaryOpEq => {
                        write!(f, "    cmp rax, rdi\n")?;
                        write!(f, "    sete al\n")?;
                        write!(f, "    movzb rax, al\n")?;
                    },
                    BinaryOpNe => {
                        write!(f, "    cmp rax, rdi\n")?;
                        write!(f, "    setne al\n")?;
                        write!(f, "    movzb rax, al\n")?;
                    },
                    BinaryOpGr => {
                        write!(f, "    cmp rax, rdi\n")?;
                        write!(f, "    setl al\n")?;
                        write!(f, "    movzb rax, al\n")?;
                    },
                    BinaryOpGe => {
                        write!(f, "    cmp rax, rdi\n")?;
                        write!(f, "    setle al\n")?;
                        write!(f, "    movzb rax, al\n")?;
                    },
                    BinaryOpAsn => {
                        match lval_size(lhs)? {
                            1 => write!(f, "    mov BYTE PTR [rax], dil\n")?,
                            2 => write!(f, "    mov WORD PTR [rax], di\n")?,
                            4 => write!(f, "    mov DWORD PTR [rax], edi\n")?,
                            8 => write!(f, "    mov QWORD PTR [rax], rdi\n")?,
                            _ => unreachable!(),
                        }
                        // Is this code needed?
                        //write!(f, "    push rdi\n")?;
                    },
                }
                write!(f, "    push rax\n")?;
            },
            Node::UnaryOperator { kind, rhs } => {
                match kind {
                    UnaryOpRf => {
                        self.gen_asm_lval(f, rhs)?;
                    }
                    UnaryOpDrf => {
                        self.gen_asm_node(f, rhs)?;
                        write!(f, "    pop rax\n")?;
                        write!(f, "    mov rax, [rax]\n")?;
                        write!(f, "    push rax\n")?;
                    }
                }
            },
            Node::LocalVariable { offset: _, ty: _ } => {
                self.gen_asm_lval(f, node)?;
                write!(f, "    pop rax\n")?;
                match lval_size(node)? {
                    1 => write!(f, "    mov al, BYTE PTR [rax]\n")?,
                    2 => write!(f, "    mov ax, WORD PTR [rax]\n")?,
                    4 => write!(f, "    mov eax, DWORD PTR [rax]\n")?,
                    8 => write!(f, "    mov rax, QWORD PTR [rax]\n")?,
                    _ => unreachable!(),
                }
                write!(f, "    push rax\n")?;
            },
            Node::GlobalVariable { name: _, offset: _, ty: _ } => {
                self.gen_asm_lval(f, node)?;
                write!(f, "    pop rax\n")?;
                match lval_size(node)? {
                    1 => write!(f, "    mov al, BYTE PTR [rax]\n")?,
                    2 => write!(f, "    mov ax, WORD PTR [rax]\n")?,
                    4 => write!(f, "    mov eax, DWORD PTR [rax]\n")?,
                    8 => write!(f, "    mov rax, QWORD PTR [rax]\n")?,
                    _ => unreachable!(),
                }
                write!(f, "    push rax\n")?;
            },
            Node::DeclareGlobal { name, size, ty: _ } => {
                write!(f, ".bss\n")?;
                write!(f, ".global {}\n", name)?;
                write!(f, "{}:\n", name)?;
                write!(f, "    .zero {}\n", size)?;
                write!(f, "\n")?;
            },
            Node::Block { nodes } => {
                self.gen_asm_block(f, nodes)?;
            },
            Node::Function { name, args, stack, block } => {
                write!(f, ".text\n")?;
                write!(f, ".global {}\n", name)?;
                write!(f, "{}:\n", name)?;

                write!(f, "    push rbp\n")?;
                write!(f, "    mov rbp, rsp\n")?;
                write!(f, "    sub rsp, {}\n", stack)?;

                let mut iter = args.into_iter().enumerate();
                while let Some((cnt, node)) = iter.next() {
                    self.gen_asm_lval(f, node)?;
                    write!(f, "    pop rax\n")?;
                    match lval_size(node)? {
                        1 => write!(f, "    mov BYTE PTR [rax], {}\n", ARG_REGS_8[cnt])?,
                        2 => write!(f, "    mov WORD PTR [rax], {}\n", ARG_REGS_16[cnt])?,
                        4 => write!(f, "    mov DWORD PTR [rax], {}\n", ARG_REGS_32[cnt])?,
                        8 => write!(f, "    mov QWORD PTR [rax], {}\n", ARG_REGS_64[cnt])?,
                        _ => unreachable!(),
                    }
                }

                self.gen_asm_node(f, block)?;

                write!(f, "\n")?;
            },
            Node::Call { name, args } => {
                let mut iter = args.into_iter().enumerate();
                while let Some((cnt, node)) = iter.next() {
                    self.gen_asm_node(f, node)?;
                    write!(f, "    pop rax\n")?;
                    write!(f, "    mov {}, rax\n", ARG_REGS_64[cnt])?;
                }

                write!(f, "    call {}\n", name)?;
                write!(f, "    push rax\n")?;
            },
            Node::If { cond, ibody } => {
                let lcnt = self.label_count;
                self.label_count += 1;

                self.gen_asm_node(f, cond)?;
                write!(f, "    pop rax\n")?;
                write!(f, "    cmp rax, 0\n")?;
                write!(f, "    je  .Lend{}\n", lcnt)?;
                self.gen_asm_node(f, ibody)?;
                write!(f, ".Lend{}:\n", lcnt)?;
            },
            Node::IfElse { cond, ibody, ebody } => {
                let lcnt = self.label_count;
                self.label_count += 1;

                self.gen_asm_node(f, cond)?;
                write!(f, "    pop rax\n")?;
                write!(f, "    cmp rax, 0\n")?;
                write!(f, "    je  .Lelse{}\n", lcnt)?;
                self.gen_asm_node(f, ibody)?;
                write!(f, "    jmp  .Lend{}\n", lcnt)?;
                write!(f, ".Lelse{}:\n", lcnt)?;
                self.gen_asm_node(f, ebody)?;
                write!(f, ".Lend{}:\n", lcnt)?;
            },
            Node::While { cond, body } => {
                let lcnt = self.label_count;
                self.label_count += 1;

                write!(f, ".Lbegin{}:\n", lcnt)?;
                self.gen_asm_node(f, cond)?;
                write!(f, "    pop rax\n")?;
                write!(f, "    cmp rax, 0\n")?;
                write!(f, "    je  .Lend{}\n", lcnt)?;
                self.gen_asm_node(f, body)?;
                write!(f, "    jmp  .Lbegin{}\n", lcnt)?;
                write!(f, ".Lend{}:\n", lcnt)?;
            },
            Node::Return { rhs } => {
                self.gen_asm_node(f, rhs)?;
                write!(f, "    pop rax\n")?;
                write!(f, "    mov rsp, rbp\n")?;
                write!(f, "    pop rbp\n")?;
                write!(f, "    ret\n")?;
            },
        }

        Ok(())
    }

    pub fn gen_asm(&mut self, f: &mut File, nodes: &Vec<Box<Node>>) -> Result<(), AsmError> {
        write!(f, ".intel_syntax noprefix\n")?;

        for node in nodes.into_iter() {
            self.gen_asm_node(f, node)?;
        }

        Ok(())
    }

    pub fn new() -> Self {
        AsmGenerator {
            label_count: 0,
        }
    }
}
