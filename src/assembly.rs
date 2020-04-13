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

#[derive(Debug)]
pub enum AsmError {
    Io(io::Error),
    Context,
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Io(e) => write!(f, "IO error! ({})", e),
            Context => write!(f, "Lvalue is invalid!"),
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

fn get_base_type(node: &Box<Node>) -> &Type {
        match &**node {
            Node::LocalVariable { offset: _, ty } => {
                if let Type::Ptr(base_ty) = ty {
                    base_ty
                } else {
                    unreachable!();
                }
            },
            Node::GlobalVariable { name: _, offset: _, ty } => {
                if let Type::Ptr(base_ty) = ty {
                    base_ty
                } else {
                    unreachable!();
                }
            },
            Node::UnaryOperator { kind, node } => {
                match kind {
                    UnaryOpDrf => get_base_type(&node),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
}

fn get_size(node: &Box<Node>) -> usize {
        match &**node {
            Node::LocalVariable { offset: _, ty } => {
                type_len(&ty)
            },
            Node::GlobalVariable { name: _, offset: _, ty } => {
                type_len(&ty)
            },
            Node::UnaryOperator { kind, node } => {
                match kind {
                    UnaryOpDrf => {
                        let ty = get_base_type(&node);
                        type_len(&ty)
                    }
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
}

pub struct AsmGenerator {
    label_count: usize,
}

impl AsmGenerator {
    fn gen_asm_block(&mut self, f: &mut File, nodes: Vec<Box<Node>>) -> Result<(), AsmError> {
        let mut iter = nodes.into_iter();
        while let Some(node) = iter.next() {
            self.gen_asm_node(f, node)?;
        }
        Ok(())
    }

    fn gen_asm_lval(&mut self, f: &mut File, node: Box<Node>) -> Result<(), AsmError> {
        match *node {
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
            Node::UnaryOperator { kind, node } => {
                match kind {
                    UnaryOpDrf => {
                        self.gen_asm_node(f, node)?;
                        Ok(())
                    }
                    _ => Err(Context),
                }
            },
            _ => Err(Context),
        }
    }

    fn gen_asm_node(&mut self, f: &mut File, node: Box<Node>) -> Result<(), AsmError> {
        match *node {
            Node::Number { val } => {
                write!(f, "    push {}\n", val)?;
            },
            Node::BinaryOperator { kind, lhs, rhs } => {
                let mut size = 0;
                if kind == BinaryOpAsn {
                    size = get_size(&lhs);
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
                        if size == 1 {
                            write!(f, "    mov BYTE PTR [rax], dil\n")?;
                        } else if size == 2 {
                            write!(f, "    mov WORD PTR [rax], di\n")?;
                        } else if size == 4 {
                            write!(f, "    mov DWORD PTR [rax], edi\n")?;
                        } else if size == 8 {
                            write!(f, "    mov QWORD PTR [rax], rdi\n")?;
                        } else {
                            unreachable!();
                        }
                        // Is this code needed?
                        //write!(f, "    push rdi\n")?;
                    },
                }
                write!(f, "    push rax\n")?;
            },
            Node::UnaryOperator { kind, node } => {
                match kind {
                    UnaryOpRf => {
                        self.gen_asm_lval(f, node)?;
                    }
                    UnaryOpDrf => {
                        self.gen_asm_node(f, node)?;
                        write!(f, "    pop rax\n")?;
                        write!(f, "    mov rax, [rax]\n")?;
                        write!(f, "    push rax\n")?;
                    }
                }
            },
            Node::LocalVariable { offset: _, ty: _ } => {
                let size = get_size(&node);
                self.gen_asm_lval(f, node)?;
                write!(f, "    pop rax\n")?;
                if size == 1 {
                    write!(f, "    mov al, BYTE PTR [rax]\n")?;
                } else if size == 2 {
                    write!(f, "    mov ax, WORD PTR [rax]\n")?;
                } else if size == 4 {
                    write!(f, "    mov eax, DWORD PTR [rax]\n")?;
                } else if size == 8 {
                    write!(f, "    mov rax, QWORD PTR [rax]\n")?;
                } else {
                    unreachable!();
                }
                write!(f, "    push rax\n")?;
            },
            Node::GlobalVariable { name: _, offset: _, ty: _ } => {
                let size = get_size(&node);
                self.gen_asm_lval(f, node)?;
                write!(f, "    pop rax\n")?;
                if size == 1 {
                    write!(f, "    mov al, BYTE PTR [rax]\n")?;
                } else if size == 2 {
                    write!(f, "    mov ax, WORD PTR [rax]\n")?;
                } else if size == 4 {
                    write!(f, "    mov eax, DWORD PTR [rax]\n")?;
                } else if size == 8 {
                    write!(f, "    mov rax, QWORD PTR [rax]\n")?;
                } else {
                    unreachable!();
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
                let arg_regs_64 = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                let arg_regs_32 = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
                let arg_regs_16 = ["di", "si", "dx", "cx", "r8d", "r9d"];
                let arg_regs_8 = ["dil", "sil", "dl", "cl", "r8d", "r9d"];
                while let Some((cnt, node)) = iter.next() {
                    let size = get_size(&node);
                    self.gen_asm_lval(f, node)?;
                    write!(f, "    pop rax\n")?;
                    if size == 1 {
                        write!(f, "    mov BYTE PTR [rax], {}\n", arg_regs_8[cnt])?;
                    } else if size == 2 {
                        write!(f, "    mov WORD PTR [rax], {}\n", arg_regs_16[cnt])?;
                    } else if size == 4 {
                        write!(f, "    mov DWORD PTR [rax], {}\n", arg_regs_32[cnt])?;
                    } else if size == 8 {
                        write!(f, "    mov QWORD PTR [rax], {}\n", arg_regs_64[cnt])?;
                    } else {
                        unreachable!();
                    }
                }

                self.gen_asm_node(f, block)?;

                write!(f, "\n")?;
            },
            Node::Call { name, args } => {
                let mut iter = args.into_iter().enumerate();
                let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                while let Some((cnt, node)) = iter.next() {
                    self.gen_asm_node(f, node)?;
                    write!(f, "    pop rax\n")?;
                    write!(f, "    mov {}, rax\n", arg_regs[cnt])?;
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

    pub fn gen_asm(&mut self, f: &mut File, nodes: Vec<Box<Node>>) -> Result<(), AsmError> {
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
