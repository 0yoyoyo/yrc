extern crate ruscom;

use std::str;
use std::fs;
use std::io::prelude::*;
use std::process::Command;

use rand::prelude::*;

use ruscom::compiler_main;

fn random_string(len: usize) -> String {
    let source = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                   abcdefghijklmnopqrstuvwxyz\
                   0123456789";
    let mut rng = rand::thread_rng();

    String::from_utf8(
        source.choose_multiple(&mut rng, len)
            .cloned()
            .collect()
    ).unwrap()
}

fn check_return_num(source_code: &str, expect: u8) {
    let output_file = format!("tests/tmp{}", random_string(8));
    let input_file = format!("{}.rs", output_file);
    let mut f = fs::File::create(&input_file).unwrap();
    write!(f, "{}", source_code).unwrap();
    println!("{}", source_code);

    let args = vec!["compiler".to_string(),
                    input_file.clone(),
                    "-o".to_string(),
                    output_file.clone()];
    compiler_main(args);

    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("./tests/run.sh {}", output_file))
        .output()
        .unwrap();
    let answer = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    fs::remove_file(&input_file).unwrap();
    fs::remove_file(&output_file).unwrap();
    println!(" -> {} (expected: {})", answer, expect);
    assert_eq!(expect, answer);
}

#[test]
fn calc_unary() {
    check_return_num("fn main() { return 0; }", 0);
    check_return_num("fn main() { return 123; }", 123);
    check_return_num("fn main() { return (123); }", 123);
}

#[test]
fn calc_binary() {
    check_return_num("fn main() { return 1 + 2; }", 3);
    check_return_num("fn main() { return 3 - 2; }", 1);
    check_return_num("fn main() { return 2 * 3; }", 6);
    check_return_num("fn main() { return 6 / 2; }", 3);
    check_return_num("fn main() { return 7 == 7; }", 1);
    check_return_num("fn main() { return 7 == 8; }", 0);
    check_return_num("fn main() { return 7 != 7; }", 0);
    check_return_num("fn main() { return 7 != 8; }", 1);
    check_return_num("fn main() { return 7 < 8; }", 1);
    check_return_num("fn main() { return 7 <= 7; }", 1);
    check_return_num("fn main() { return 7 <= 8; }", 1);
    check_return_num("fn main() { return 7 < 7; }", 0);
    check_return_num("fn main() { return 7 <= 6; }", 0);
    check_return_num("fn main() { return 7 <= 6; }", 0);
    check_return_num("fn main() { return 8 > 7; }", 1);
    check_return_num("fn main() { return 7 >= 7; }", 1);
    check_return_num("fn main() { return 8 >= 7; }", 1);
    check_return_num("fn main() { return 7 > 7; }", 0);
    check_return_num("fn main() { return 6 >= 7; }", 0);
    check_return_num("fn main() { return 6 >= 7; }", 0);
}

#[test]
fn calc_bool() {
    check_return_num("fn main() {\
                          let a: bool;\
                          a = true;\
                          if a {\
                              return 2;\
                          }\
                          return 3;\
                      }", 2);
    check_return_num("fn main() {\
                          let a: bool;\
                          a = false;\
                          if a {\
                              return 2;\
                          }\
                          return 3;\
                      }", 3);
}

#[test]
fn calc_combination() {
    check_return_num("fn main() { return -1 + 2; }", 1);
    check_return_num("fn main() { return -(1 + 2) + 4; }", 1);
    check_return_num("fn main() { return 2 * 3 + 6 / 2; }", 9);
    check_return_num("fn main() { return 2 * (3 + 6) / 3; }", 6);
}

#[test]
fn calc_local_variable() {
    check_return_num("fn main() {\
                          let a: i32;\
                          a = 1;\
                          return a;\
                      }", 1);
    check_return_num("fn main() {\
                          let z: i32;\
                          z = 1;\
                          return z;\
                      }", 1);
    check_return_num("fn main() {\
                          let n: i32;\
                          n = 10 + 2;\
                          return n * 2;\
                      }", 24);
    check_return_num("fn main() {\
                          let abc: i32;\
                          let def: i32;\
                          abc = 2;\
                          def = 3;\
                          return abc + def;\
                      }", 5);
    check_return_num("fn main() {\
                          let abc: i32;\
                          let def: i32;\
                          abc = 2;\
                          def = abc + 3;\
                          return def;\
                      }", 5);
}

#[test]
fn calc_type() {
    check_return_num("fn main() {\
                          let a: i8;\
                          a = 1;\
                          return a;\
                      }", 1);
    check_return_num("fn main() {\
                          let a: i16;\
                          a = 1;\
                          return a;\
                      }", 1);
    check_return_num("fn main() {\
                          let a: i32;\
                          a = 1;\
                          return a;\
                      }", 1);
    check_return_num("fn main() {\
                          let a: i64;\
                          a = 1;\
                          return a;\
                      }", 1);
    // To check upper bits are cleared.
    check_return_num("static a: i8;\
                      fn main() {\
                          let b: i8;\
                          a = 1;\
                          b = 1;\
                          return a == b;\
                      }", 1);
    check_return_num("static a: i16;\
                      fn main() {\
                          let b: i16;\
                          a = 1;\
                          b = 1;\
                          return a == b;\
                      }", 1);
}

#[test]
fn calc_global_variable() {
    check_return_num("static a: i32;\
                      fn main() {\
                          a = 1;\
                          return a;\
                      }", 1);
    check_return_num("static a: [i32; 10];\
                      fn main() {\
                          a[8] = 1;\
                          a[9] = 2;\
                          return a[8] + a[9];\
                      }", 3);
    check_return_num("static a: i32;\
                      fn main() {\
                          let b: i32;\
                          a = 1;\
                          b = 2;\
                          return a + b;\
                      }", 3);
    check_return_num("static a: [i32; 2];\
                      static b: [i32; 2];\
                      fn main() {\
                          a[1] = 1;\
                          b[0] = 2;\
                          return a[1] + b[0];\
                      }", 3);
    check_return_num("static a: [i8; 4];\
                      static b: i32;\
                      fn main() {\
                          b = 2;\
                          a[3] = 1;\
                          return a[3] + b;\
                      }", 3);
}

#[test]
fn calc_control() {
    check_return_num("fn main() {\
                          let a: i32;\
                          a = 1;\
                          if 1 == 1 {\
                              a = 2;\
                          } else {\
                              a = 3;\
                          }\
                          return a;\
                      }", 2);
    check_return_num("fn main() {\
                          let a: i32;\
                          a = 1;\
                          if 1 == 2 {\
                              a = 2;\
                          } else {\
                              a = 3;\
                          }\
                          return a;\
                      }", 3);
    check_return_num("fn main() {\
                          let a: i32;\
                          let b: i32;\
                          a = 1;\
                          if 1 == 1 {\
                              b = 1;\
                              a = b + 1;\
                          }\
                          return a;\
                      }", 2);
    check_return_num("fn main() {\
                          let a: i32;\
                          a = 1;\
                          if 1 == 1 {\
                              a = a + 1;\
                          }\
                          if 1 == 2 {\
                              a = a + 1;\
                          }\
                          if 2 == 2 {\
                              a = a + 1;\
                              if 3 == 3 {\
                                  a = a + 1;\
                              }\
                          }\
                          return a;\
                      }", 4);
    check_return_num("fn main() {\
                          let a: i32;\
                          a = 1;\
                          while a != 10 {\
                              a = a + 1;\
                          }\
                          return a;\
                      }", 10);
}

#[test]
fn calc_func() {
    check_return_num("fn foo() {\
                          return 3;\
                      }\
                      fn main() {\
                          return foo();\
                      }", 3);
    check_return_num("fn foo() {\
                          let c: i32;\
                          let d: i32;\
                          c = 3;\
                          d = 4;\
                          return c + d;\
                      }\
                      fn main() {\
                          let a: i32;\
                          let b: i32;\
                          a = 1;\
                          b = 2;\
                          return a + b + foo();\
                      }", 10);
    check_return_num("fn foo() {\
                          let a: i32;\
                          let b: i32;\
                          a = 3;\
                          b = 4;\
                          return a + b;\
                      }\
                      fn main() {\
                          let a: i32;\
                          let b: i32;\
                          a = 1;\
                          b = 2;\
                          return a + b + foo();\
                      }", 10);
    check_return_num("fn foo(a: i32) {\
                          return a * 2;\
                      }\
                      fn main() {\
                          return foo(2);\
                      }", 4);
    check_return_num("fn foo(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) {\
                          return (a + b + c + d + e + f) * 2;\
                      }\
                      fn main() {\
                          return foo(1, 2, 3, 4, 5, 6);\
                      }", 42);
}

#[test]
fn calc_reference() {
    check_return_num("fn main() {\
                          let a: i32;\
                          let b: &i32;\
                          a = 2;\
                          b = &a;\
                          return *b;\
                      }", 2);
    check_return_num("fn foo() {\
                          let a: i32;\
                          let b: &i32;\
                          b = &a;\
                          *b = 3;\
                          return a;\
                      }\
                      fn main() {\
                          return foo();\
                      }", 3);
}

#[test]
fn calc_array() {
    check_return_num("fn main() {\
                          let a: [i32; 10];\
                          a[0] = 1;\
                          a[1] = 2;\
                          a[2] = 3;\
                          return a[0] + a[1] + a[2];\
                      }", 6);
    check_return_num("fn foo() {\
                          let a: [i32; 4];\
                          a[2] = 3;\
                          return a[2];\
                      }\
                      fn main() {\
                          return foo();\
                      }", 3);
    check_return_num("fn main() {\
                          let a: [i32; 4];\
                          let b: [i32; 4];\
                          a[3] = 2;\
                          b[3] = 3;\
                          return a[3] + b[3];\
                      }", 5);
}

#[test]
fn check_comment() {
    check_return_num("fn main() {\
                          // This is\n\
                          // one line\n\
                          // comment.\n\
                          return 1;\
                      }", 1);
    check_return_num("fn main() {\
                          /*\
                           * This is\
                           * multiple line\
                           * comment.\
                           */\
                          return 1;\
                      }", 1);
    check_return_num("fn main() {\
                          /* No content */\
                          /**/\
                          return 1;\
                      }", 1);
}

#[test]
fn check_format() {
    check_return_num("fn main() { return 1+2+3; }", 6);
    check_return_num("fn main() { return  1 + 2 + 3 ; }", 6);
    check_return_num("fn main() { return 1 +  2   +    3; }", 6);
    check_return_num("fn main() { return (1+2)+3; }", 6);
    check_return_num("fn main() { return 1+(2+3); }", 6);
    check_return_num("fn main() { return (1+2+3); }", 6);
}
