use std::env;

mod token;
// use token::*;

mod error;
use error::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("USAGE: rick <filename>");
        std::process::exit(1);
    }

    error::set_source(args[1].clone());

}
