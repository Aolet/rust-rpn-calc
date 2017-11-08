
pub mod calculator;

use std::io::{self, BufRead, Write};
use std::iter::repeat;
use calculator::Calculator;

fn main() {

    println!("");
    println!("Welcome to simplecalc, a Reverse Polish Notation (RPN) calculator!");
    println!("For help, please use the 'help' command!");
    println!("Use Ctrl+D to exit the calculator at any time.");
    println!("");

    let stdin = io::stdin();

    let prompt_writer = repeat(())
        .map(|_| {
            print!("> ");
            io::stdout()
                .flush()
                .expect("Error flushing output stream");
        });
    
    let lines = prompt_writer
        .zip(
            stdin
                .lock()
                .lines()
                .map(|x| String::from(x.expect("Error reading user input"))))
        .map(|(_, line)| line);

    let words = lines.flat_map(|line| line
        .split_whitespace()
        .map(|x| String::from(x))
        .collect::<Vec<String>>()
        .into_iter());

    words.fold(Calculator::new(), |calc, word| {
        let (calc, output) = calc.exec(word);
        output
            .into_iter()
            .for_each(|s| {
                println!("{}", s);
            });
        calc
    });

    println!("\ngoodbyte!")
}
