.intel_syntax noprefix
.section .rodata
.LC0:
    .ascii "Hello"
.text
.global foo
foo:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov rax, rbp
    sub rax, 16
    push rax
    pop rax
