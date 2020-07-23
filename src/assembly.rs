use std::fmt;
use std::io;
use std::fs::File;
use std::io::prelude::*;

use super::parse::Node;
use super::parse::BinaryOpKind::*;
use super::parse::UnaryOpKind::*;
use super::parse::Type;
use super::parse::type_size;

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

fn is_call(node: &Box<Node>) -> bool {
    match &**node {
        Node::Call { name: _, args: _, ty: _ } => true,
        _ => false,
    }
}

fn is_slice(node: &Box<Node>) -> bool {
    if let Ok(ty) = lval_type(node) {
        match ty {
            Type::Slc(_) => true,
            _ => false,
        }
    } else {
        false
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
    Ok(type_size(ty))
}

pub struct AsmGenerator {
    label_count: usize,
}

impl AsmGenerator {
    fn gen_asm_call(&mut self, f: &mut File, node: &Box<Node>) -> Result<(), AsmError> {
        match &**node {
            Node::Call { name, args, ty: _ } => {
                let mut swap = false;
                let iter = args.iter().enumerate();
                let mut offset = 0;
                for (cnt, node) in iter {
                    let index = cnt + offset;
                    if is_slice(node) {
                        self.gen_asm_lval(f, node)?;
                        writeln!(f, "    pop rax")?;
                        writeln!(f, "    mov {}, QWORD PTR [rax]", ARG_REGS_64[index])?;
                        writeln!(f, "    mov {}, QWORD PTR [rax+8]", ARG_REGS_64[index + 1])?;
                        offset = offset + 1;
                    } else {
                        self.gen_asm_node(f, node)?;
                        writeln!(f, "    pop rax")?;

                        // Temporarily use r10 register because above gen_asm_node()
                        // can break rdi register.
                        if index == 0 {
                            swap = true;
                            writeln!(f, "    mov r10, rax")?;
                        } else {
                            writeln!(f, "    mov {}, rax", ARG_REGS_64[index])?;
                        }
                    }
                }

                if swap {
                    writeln!(f, "    mov rdi, r10")?;
                }
                writeln!(f, "    call {}@PLT", name)?;
                Ok(())
            },
            _ => unreachable!(),
        }
    }

    fn gen_asm_lval(&mut self, f: &mut File, node: &Box<Node>) -> Result<(), AsmError> {
        match &**node {
            Node::LocalVariable { offset, ty: _ } => {
                writeln!(f, "    mov rax, rbp")?;
                writeln!(f, "    sub rax, {}", offset)?;
                writeln!(f, "    push rax")?;
                Ok(())
            },
            Node::GlobalVariable { name, offset, ty: _ } => {
                writeln!(f, "    lea rax, QWORD PTR {}[rip+{}]", name, offset)?;
                writeln!(f, "    push rax")?;
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
                writeln!(f, "    push {}", val)?;
            },
            Node::Bool { bl } => {
                if *bl {
                    writeln!(f, "    push 1")?;
                } else {
                    writeln!(f, "    push 0")?;
                }
            },
            Node::StrLiteral { s, label } => {
                writeln!(f, "    lea rax, QWORD PTR .LC{}[rip]", label)?;
                writeln!(f, "    push rax")?;
                writeln!(f, "    push {}", s.len())?;
            },
            Node::BinaryOperator { kind, lhs, rhs } => {
                if *kind == BinaryOpAsn {
                    self.gen_asm_lval(f, lhs)?;
                } else {
                    self.gen_asm_node(f, lhs)?;
                }
                self.gen_asm_node(f, rhs)?;
                if is_slice(lhs) {
                    writeln!(f, "    pop rdx")?;
                    writeln!(f, "    pop rdi")?;
                } else {
                    writeln!(f, "    pop rdi")?;
                }
                writeln!(f, "    pop rax")?;
                match kind {
                    BinaryOpAdd => {
                        writeln!(f, "    add rax, rdi")?;
                    },
                    BinaryOpSub => {
                        writeln!(f, "    sub rax, rdi")?;
                    },
                    BinaryOpMul => {
                        writeln!(f, "    imul rax, rdi")?;
                    },
                    BinaryOpDiv => {
                        writeln!(f, "    cqo")?;
                        writeln!(f, "    idiv rdi")?;
                    },
                    BinaryOpEq => {
                        writeln!(f, "    cmp rax, rdi")?;
                        writeln!(f, "    sete al")?;
                        writeln!(f, "    movzb rax, al")?;
                    },
                    BinaryOpNe => {
                        writeln!(f, "    cmp rax, rdi")?;
                        writeln!(f, "    setne al")?;
                        writeln!(f, "    movzb rax, al")?;
                    },
                    BinaryOpGr => {
                        writeln!(f, "    cmp rax, rdi")?;
                        writeln!(f, "    setl al")?;
                        writeln!(f, "    movzb rax, al")?;
                    },
                    BinaryOpGe => {
                        writeln!(f, "    cmp rax, rdi")?;
                        writeln!(f, "    setle al")?;
                        writeln!(f, "    movzb rax, al")?;
                    },
                    BinaryOpAsn => {
                        if is_slice(lhs) {
                            writeln!(f, "    mov QWORD PTR [rax], rdi")?;
                            writeln!(f, "    mov QWORD PTR [rax+8], rdx")?;
                        } else {
                            match lval_size(lhs)? {
                                1 => writeln!(f, "    mov BYTE PTR [rax], dil")?,
                                2 => writeln!(f, "    mov WORD PTR [rax], di")?,
                                4 => writeln!(f, "    mov DWORD PTR [rax], edi")?,
                                8 => writeln!(f, "    mov QWORD PTR [rax], rdi")?,
                                _ => unreachable!(),
                            }
                        }
                    },
                }
                if *kind != BinaryOpAsn {
                    writeln!(f, "    push rax\n")?;
                }
            },
            Node::UnaryOperator { kind, rhs } => {
                match kind {
                    UnaryOpRf => {
                        self.gen_asm_lval(f, rhs)?;
                    }
                    UnaryOpDrf => {
                        self.gen_asm_node(f, rhs)?;
                        writeln!(f, "    pop rax")?;
                        writeln!(f, "    mov rax, QWORD PTR [rax]")?;
                        writeln!(f, "    push rax")?;
                    }
                }
            },
            Node::LocalVariable { offset: _, ty: _ } => {
                self.gen_asm_lval(f, node)?;
                writeln!(f, "    pop rax\n")?;
                if is_slice(node) {
                    writeln!(f, "    mov rdi, QWORD PTR [rax]")?;
                    writeln!(f, "    mov rax, QWORD PTR [rax+8]")?;
                    writeln!(f, "    push rdi")?;
                    writeln!(f, "    push rax")?;
                } else {
                    match lval_size(node)? {
                        1 => writeln!(f, "    movsx eax, BYTE PTR [rax]")?,
                        2 => writeln!(f, "    movsx eax, WORD PTR [rax]")?,
                        4 => writeln!(f, "    mov eax, DWORD PTR [rax]")?,
                        8 => writeln!(f, "    mov rax, QWORD PTR [rax]")?,
                        _ => unreachable!(),
                    }
                    writeln!(f, "    push rax")?;
                }
            },
            Node::DeclareLocal { offset: _, ty: _ } => {
                // Do nothing
            },
            Node::GlobalVariable { name: _, offset: _, ty: _ } => {
                self.gen_asm_lval(f, node)?;
                writeln!(f, "    pop rax\n")?;
                if is_slice(node) {
                    writeln!(f, "    mov rdi, QWORD PTR [rax]")?;
                    writeln!(f, "    mov rax, QWORD PTR [rax+8]")?;
                    writeln!(f, "    push rdi")?;
                    writeln!(f, "    push rax")?;
                } else {
                    match lval_size(node)? {
                        1 => writeln!(f, "    movsx eax, BYTE PTR [rax]")?,
                        2 => writeln!(f, "    movsx eax, WORD PTR [rax]")?,
                        4 => writeln!(f, "    mov eax, DWORD PTR [rax]")?,
                        8 => writeln!(f, "    mov rax, QWORD PTR [rax]")?,
                        _ => unreachable!(),
                    }
                    writeln!(f, "    push rax")?;
                }
            },
            Node::DeclareGlobal { name, size, ty: _ } => {
                writeln!(f, ".bss")?;
                writeln!(f, ".global {}", name)?;
                writeln!(f, "{}:", name)?;
                writeln!(f, "    .zero {}", size)?;
                writeln!(f)?;
            },
            Node::Block { nodes } => {
                self.gen_asm_node_stream(f, nodes)?;
            },
            Node::Function { name, args, stack, block } => {
                writeln!(f, ".text")?;
                writeln!(f, ".global {}", name)?;
                writeln!(f, "{}:", name)?;

                writeln!(f, "    push rbp")?;
                writeln!(f, "    mov rbp, rsp")?;
                writeln!(f, "    sub rsp, {}", stack)?;

                let iter = args.iter().enumerate();
                let mut offset = 0;
                for (cnt, node) in iter {
                    let index = cnt + offset;
                    self.gen_asm_lval(f, node)?;
                    writeln!(f, "    pop rax")?;
                    if is_slice(node) {
                        writeln!(f, "    mov QWORD PTR [rax], {}", ARG_REGS_64[index])?;
                        writeln!(f, "    mov QWORD PTR [rax+8], {}", ARG_REGS_64[index+1])?;
                        offset = offset + 1;
                    } else {
                        match lval_size(node)? {
                            1 => writeln!(f, "    mov BYTE PTR [rax], {}", ARG_REGS_8[index])?,
                            2 => writeln!(f, "    mov WORD PTR [rax], {}", ARG_REGS_16[index])?,
                            4 => writeln!(f, "    mov DWORD PTR [rax], {}", ARG_REGS_32[index])?,
                            8 => writeln!(f, "    mov QWORD PTR [rax], {}", ARG_REGS_64[index])?,
                            _ => unreachable!(),
                        }
                    }
                }

                self.gen_asm_node(f, block)?;

                writeln!(f)?;
            },
            Node::DeclareFunc { name: _, args: _ } => {
                // Do nothing
            }
            Node::Call { name: _, args: _, ty } => {
                self.gen_asm_call(f, node)?;
                if let Type::Slc(_) = ty {
                    writeln!(f, "    push rdx")?;
                    writeln!(f, "    push rax")?;
                } else {
                    writeln!(f, "    push rax")?;
                }
            },
            Node::If { cond, ibody } => {
                let lcnt = self.label_count;
                self.label_count += 1;

                self.gen_asm_node(f, cond)?;
                writeln!(f, "    pop rax")?;
                writeln!(f, "    cmp rax, 0")?;
                writeln!(f, "    je  .Lend{}", lcnt)?;
                self.gen_asm_node(f, ibody)?;
                writeln!(f, ".Lend{}:", lcnt)?;
            },
            Node::IfElse { cond, ibody, ebody } => {
                let lcnt = self.label_count;
                self.label_count += 1;

                self.gen_asm_node(f, cond)?;
                writeln!(f, "    pop rax")?;
                writeln!(f, "    cmp rax, 0")?;
                writeln!(f, "    je  .Lelse{}", lcnt)?;
                self.gen_asm_node(f, ibody)?;
                writeln!(f, "    jmp  .Lend{}", lcnt)?;
                writeln!(f, ".Lelse{}:", lcnt)?;
                self.gen_asm_node(f, ebody)?;
                writeln!(f, ".Lend{}:", lcnt)?;
            },
            Node::While { cond, body } => {
                let lcnt = self.label_count;
                self.label_count += 1;

                writeln!(f, ".Lbegin{}:", lcnt)?;
                self.gen_asm_node(f, cond)?;
                writeln!(f, "    pop rax")?;
                writeln!(f, "    cmp rax, 0")?;
                writeln!(f, "    je  .Lend{}", lcnt)?;
                self.gen_asm_node(f, body)?;
                writeln!(f, "    jmp  .Lbegin{}", lcnt)?;
                writeln!(f, ".Lend{}:", lcnt)?;
            },
            Node::Return { rhs, ty } => {
                self.gen_asm_node(f, rhs)?;
                if let Type::Slc(_) = ty {
                    writeln!(f, "    pop rax")?;
                    writeln!(f, "    pop rdx")?;
                } else {
                    writeln!(f, "    pop rax")?;
                }
                writeln!(f, "    mov rsp, rbp")?;
                writeln!(f, "    pop rbp")?;
                writeln!(f, "    ret")?;
            },
        }

        Ok(())
    }

    fn gen_asm_node_stream(&mut self, f: &mut File, nodes: &Vec<Box<Node>>) -> Result<(), AsmError> {
        let iter = nodes.iter();
        for node in iter {
            if is_call(node) {
                // Do not handle return value when a function is called alone.
                self.gen_asm_call(f, node)?;
            } else {
                self.gen_asm_node(f, node)?;
            }
        }
        Ok(())
    }

    pub fn gen_asm(&mut self, f: &mut File, nodes: &Vec<Box<Node>>, literals: &Vec<String>) -> Result<(), AsmError> {
        writeln!(f, ".intel_syntax noprefix")?;

        writeln!(f, ".section .rodata")?;
        let iter = literals.iter().enumerate();
        for (cnt, lit) in iter {
            writeln!(f, ".LC{}:", cnt)?;
            writeln!(f, "    .ascii \"{}\"", lit)?;
        }

        self.gen_asm_node_stream(f, nodes)?;

        Ok(())
    }

    pub fn new() -> Self {
        AsmGenerator {
            label_count: 0,
        }
    }
}
