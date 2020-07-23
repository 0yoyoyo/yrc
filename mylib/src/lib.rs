#[no_mangle]
fn myprint(s: &str) {
    println!("{}", s);
}

#[no_mangle]
fn myprint2(_a: i32, s: &str, _b: i32) {
    println!("{}", s);
}

#[no_mangle]
fn myadd(a: i32, b: i32) -> i32 {
    a + b
}
