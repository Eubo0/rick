use std::env;

mod token;
// use token::*;

mod scanner;
use scanner::*;

mod error;
use error::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: add more options
    if args.len() != 2 {
        eprintln!("USAGE: rick <filename>");
        std::process::exit(1);
    }

    error::set_source(args[1].clone());
    error::set_loc(1, 0);

    let mut scanner: Scanner = Scanner::new(args[1].clone());
    
    scanner.scan_source();

    println!("{:#?}", scanner.tok_stream);
}
