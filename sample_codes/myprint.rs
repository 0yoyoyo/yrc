fn myprint(s: &str);
fn myprint2(a: i32, s: &str, b: i32);

fn print_wrapper(a: i32, s: &str, b: i32) {
    myprint2(a, s, b);
    return 0;
}

fn main() {
    let s: &str;
    s = "Hello my Rust!";
    myprint(s);
    myprint2(1, s, 2);
    print_wrapper(1, s, 2);
    return 0;
}
