# YRC
YRC is something like a Rust compiler in the making.

[![Build Status](https://travis-ci.org/0yoyoyo/yrc.svg?branch=master)](https://travis-ci.org/0yoyoyo/yrc)
[![codecov](https://codecov.io/gh/0yoyoyo/yrc/branch/master/graph/badge.svg)](https://codecov.io/gh/0yoyoyo/yrc)

# Usage

```
Options:
    -o, --output NAME   set output file name
    -s, --asm           output assemble code
    -h, --help          print this help message
```

# Supported syntax

- **Types**: _i8_, _i16_, _i32_, _i64_, _u8_, _u16_, _u32_, _u64_, _array_, _bool_, _pointer_, _reference_, _slice_, _str_
- **Controll syntax**: _if_, _else_, _while_
- **Arithmetic operation**
- **Local and global variable binding**
- **Function difinition and call**


# Syntax not yet supported

- **Pattern match**
- **Mutability**
- **Type inference**
- **Ownership**
- **Structs**
- **Tuples**
- **Implementations**
- **closure**
- **Traits**
- **Macros**

and more...
