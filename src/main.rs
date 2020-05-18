use std::env;
use ruscom::compiler_main;

fn main() {
    let args = env::args().collect();
    compiler_main(args);
}
