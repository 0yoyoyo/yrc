use std::fmt;
use std::io;
use std::fs::File;
use std::io::prelude::*;

use super::parse::Node;
use super::parse::NodeKind::*;
use super::parse::get_lvar_num;

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

fn gen_asm_block(f: &mut File, nodes: Vec<Box<Node>>) -> Result<(), AsmError> {
    let mut iter = nodes.into_iter();
    while let Some(node) = iter.next() {
        gen_asm_node(f, node)?;
    }
    Ok(())
}

fn gen_asm_lval(f: &mut File, node: Box<Node>) -> Result<(), AsmError> {
    match *node {
        Node::LocalVariable { offset } => {
            write!(f, "    mov rax, rbp\n")?;
            write!(f, "    sub rax, {}\n", offset)?;
            write!(f, "    push rax\n")?;
            Ok(())
        },
        _ => Err(Context),
    }
}

fn gen_asm_node(f: &mut File, node: Box<Node>) -> Result<(), AsmError> {
    match *node {
        Node::Number { val } => {
            write!(f, "    push {}\n", val)?;
        },
        Node::Operator { kind, lhs, rhs } => {
            if kind == NodeAsn {
                gen_asm_lval(f, lhs)?;
            } else {
                gen_asm_node(f, lhs)?;
            }
            gen_asm_node(f, rhs)?;
            write!(f, "    pop rdi\n")?;
            write!(f, "    pop rax\n")?;
            match kind {
                NodeAdd => {
                    write!(f, "    add rax, rdi\n")?;
                },
                NodeSub => {
                    write!(f, "    sub rax, rdi\n")?;
                },
                NodeMul => {
                    write!(f, "    imul rax, rdi\n")?;
                },
                NodeDiv => {
                    write!(f, "    cqo\n")?;
                    write!(f, "    idiv rdi\n")?;
                },
                NodeEq => {
                    write!(f, "    cmp rax, rdi\n")?;
                    write!(f, "    sete al\n")?;
                    write!(f, "    movzb rax, al\n")?;
                },
                NodeNe => {
                    write!(f, "    cmp rax, rdi\n")?;
                    write!(f, "    setne al\n")?;
                    write!(f, "    movzb rax, al\n")?;
                },
                NodeGr => {
                    write!(f, "    cmp rax, rdi\n")?;
                    write!(f, "    setl al\n")?;
                    write!(f, "    movzb rax, al\n")?;
                },
                NodeGe => {
                    write!(f, "    cmp rax, rdi\n")?;
                    write!(f, "    setle al\n")?;
                    write!(f, "    movzb rax, al\n")?;
                },
                NodeAsn => {
                    write!(f, "    mov [rax], rdi\n")?;
                    // Is this code needed?
                    //write!(f, "    push rdi\n")?;
                },
            }
            write!(f, "    push rax\n")?;
        },
        Node::LocalVariable { offset: _ } => {
            gen_asm_lval(f, node)?;
            write!(f, "    pop rax\n")?;
            write!(f, "    mov rax, [rax]\n")?;
            write!(f, "    push rax\n")?;
        },
        Node::Function { name, args, block } => {
            write!(f, ".global {}\n", name)?;
            write!(f, "{}:\n", name)?;

            write!(f, "    push rbp\n")?;
            write!(f, "    mov rbp, rsp\n")?;
            write!(f, "    sub rsp, {}\n", 8 * get_lvar_num())?;

            let mut iter = args.into_iter().enumerate();
            let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            while let Some((cnt, node)) = iter.next() {
                gen_asm_lval(f, node)?;
                write!(f, "    pop rax\n")?;
                write!(f, "    mov [rax], {}\n", arg_regs[cnt])?;
            }

            gen_asm_node(f, block)?;

            write!(f, "\n")?;
        },
        Node::Call { name, args } => {
            let mut iter = args.into_iter().enumerate();
            let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            while let Some((cnt, node)) = iter.next() {
                gen_asm_node(f, node)?;
                write!(f, "    pop rax\n")?;
                write!(f, "    mov {}, rax\n", arg_regs[cnt])?;
            }

            write!(f, "    call {}\n", name)?;
            write!(f, "    push rax\n")?;
        },
        Node::Block { nodes } => {
            gen_asm_block(f, nodes)?;
        },
        Node::Return { rhs } => {
            gen_asm_node(f, rhs)?;
            write!(f, "    pop rax\n")?;
            write!(f, "    mov rsp, rbp\n")?;
            write!(f, "    pop rbp\n")?;
            write!(f, "    ret\n")?;
        },
        Node::If { cond, ibody } => {
            gen_asm_node(f, cond)?;
            write!(f, "    pop rax\n")?;
            write!(f, "    cmp rax, 0\n")?;
            write!(f, "    je  .Lend\n")?;
            gen_asm_node(f, ibody)?;
            write!(f, ".Lend:\n")?;
        },
        Node::IfElse { cond, ibody, ebody } => {
            gen_asm_node(f, cond)?;
            write!(f, "    pop rax\n")?;
            write!(f, "    cmp rax, 0\n")?;
            write!(f, "    je  .Lelse\n")?;
            gen_asm_node(f, ibody)?;
            write!(f, "    jmp  .Lend\n")?;
            write!(f, ".Lelse:\n")?;
            gen_asm_node(f, ebody)?;
            write!(f, ".Lend:\n")?;
        },
        Node::While { cond, body } => {
            write!(f, ".Lbegin:\n")?;
            gen_asm_node(f, cond)?;
            write!(f, "    pop rax\n")?;
            write!(f, "    cmp rax, 0\n")?;
            write!(f, "    je  .Lend\n")?;
            gen_asm_node(f, body)?;
            write!(f, "    jmp  .Lbegin\n")?;
            write!(f, ".Lend:\n")?;
        },
    }

    Ok(())
}

pub fn gen_asm(f: &mut File, nodes: Vec<Box<Node>>) -> Result<(), AsmError> {
    write!(f, ".intel_syntax noprefix\n")?;

    for node in nodes.into_iter() {
        gen_asm_node(f, node)?;
    }

    Ok(())
}
