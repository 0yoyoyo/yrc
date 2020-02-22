use std::fs::File;
use std::io::prelude::*;

use crate::parse::Node;
use crate::parse::NodeKind::*;

fn assemble_lval(f: &mut File, node: Box<Node>) -> std::io::Result<()> {
    match *node {
        Node::LocalVariable { offset } => {
            f.write_fmt(format_args!("    mov rax, rbp\n"))?;
            f.write_fmt(format_args!("    sub rax, {}\n", offset))?;
            f.write_fmt(format_args!("    push rax\n"))?;
        },
        _ => unimplemented!(),
    }
    Ok(())
}

fn assemble_node(f: &mut File, node: Box<Node>) -> std::io::Result<()> {
    match *node {
        Node::Number {val} => {
            f.write_fmt(format_args!("    push {}\n", val))?;
        },
        Node::Operator { kind, lhs, rhs} => {
            assemble_node(f, lhs)?;
            assemble_node(f, rhs)?;
            f.write_fmt(format_args!("    pop rdi\n"))?;
            f.write_fmt(format_args!("    pop rax\n"))?;
            match kind {
                NodeAdd => {
                    f.write_fmt(format_args!("    add rax, rdi\n"))?;
                },
                NodeSub => {
                    f.write_fmt(format_args!("    sub rax, rdi\n"))?;
                },
                NodeMul => {
                    f.write_fmt(format_args!("    imul rax, rdi\n"))?;
                },
                NodeDiv => {
                    f.write_fmt(format_args!("    cqo\n"))?;
                    f.write_fmt(format_args!("    idiv rdi\n"))?;
                },
                NodeEq => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    sete al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
                NodeNe => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    setne al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
                NodeGr => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    setl al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
                NodeGe => {
                    f.write_fmt(format_args!("    cmp rax, rdi\n"))?;
                    f.write_fmt(format_args!("    setle al\n"))?;
                    f.write_fmt(format_args!("    movzb rax, al\n"))?;
                },
            }
            f.write_fmt(format_args!("    push rax\n"))?;
        },
        Node::LocalVariable { offset: _ } => {
            assemble_lval(f, node)?;
            f.write_fmt(format_args!("    pop rax\n"))?;
            f.write_fmt(format_args!("    mov rax, [rax]\n"))?;
            f.write_fmt(format_args!("    push rax\n"))?;
        },
        Node::Assignment { lhs, rhs } => {
            assemble_lval(f, lhs)?;
            assemble_node(f, rhs)?;
            f.write_fmt(format_args!("    pop rdi\n"))?;
            f.write_fmt(format_args!("    pop rax\n"))?;
            f.write_fmt(format_args!("    mov [rax], rdi\n"))?;
            f.write_fmt(format_args!("    push rdi\n"))?;
        }
    }

    Ok(())
}

pub fn assemble(f: &mut File, nodes: Vec<Box<Node>>) -> std::io::Result<()> {
    f.write_fmt(format_args!(".intel_syntax noprefix\n"))?;
    f.write_fmt(format_args!(".global main\n"))?;
    f.write_fmt(format_args!("main:\n"))?;

    f.write_fmt(format_args!("    push rbp\n"))?;
    f.write_fmt(format_args!("    mov rbp, rsp\n"))?;
    f.write_fmt(format_args!("    sub rsp, 208\n"))?;

    for node in nodes.into_iter() {
        assemble_node(f, node)?;
        f.write_fmt(format_args!("    pop rax\n"))?;
    }

    f.write_fmt(format_args!("    mov rsp, rbp\n"))?;
    f.write_fmt(format_args!("    pop rbp\n"))?;
    f.write_fmt(format_args!("    ret\n"))?;

    Ok(())
}
