use std::env;
use yrc::compiler_main;

fn main() {
    let args = env::args().collect();
    compiler_main(args);
}
