use std::io;
use std::io::{BufRead, Write};
use std::iter::repeat;
use std::collections::{VecDeque, HashMap};
use std::str::FromStr;
use std::rc::Rc;

#[derive(Clone)]
struct Calculator {
    stack: VecDeque<f64>,
    ops: HashMap<String, Rc<Fn(Calculator) -> (Calculator, Vec<String>)>>,
}

fn bool_to_f64(b: bool) -> f64 {
    if b {1.0} else {0.0}
}

fn f64_to_bool(f: f64) -> bool {
    f != 0.0
}

impl Calculator {
    pub fn new() -> Self {
        let mut calc = Calculator {
            stack: VecDeque::new(),
            ops: HashMap::new(),
        };

        let make_binop = |name: String, binop: Box<Fn(f64, f64) -> f64>|
            Rc::new(move |mut calc: Calculator| {
                let len = calc.stack.len();
                if len < 2 {
                    (calc, vec![format!("'{}' requires stack size >= 2, current = {}", name, len)])
                } else {
                    let a = calc.stack.pop_back().unwrap();
                    let b = calc.stack.pop_back().unwrap();
                    calc.stack.push_back(binop(a, b));
                    (calc, vec![])
                }
            });

        let make_unop = |name: String, unop: Box<Fn(f64) -> f64>|
            Rc::new(move |mut calc: Calculator| {
                match calc.stack.pop_back() {
                    None => (calc, vec![format!("'{}' requires a non-empty stack", name)]),
                    Some(x) => {
                        calc.stack.push_back(unop(x));
                        (calc, vec![])
                    }
                }
            });

        // binary floating point operations
        calc.ops.insert(String::from("-"),
            make_binop(String::from("Subtract"), Box::new(|a, b| a - b)));
        calc.ops.insert(String::from("+"),
            make_binop(String::from("Add"), Box::new(|a, b| a + b)));
        calc.ops.insert(String::from("*"),
            make_binop(String::from("Multiply"), Box::new(|a, b| a * b)));
        calc.ops.insert(String::from("/"),
            make_binop(String::from("Divide"), Box::new(|a, b| a / b)));
        calc.ops.insert(String::from("^"),
            make_binop(String::from("Exponentiate"), Box::new(|a, b| a.powf(b))));
        calc.ops.insert(String::from("e"),
            make_binop(String::from("xe^y"), Box::new(|a, b| a * (10.0 as f64).powf(b))));
        calc.ops.insert(String::from("log"),
            make_binop(String::from("Logarithm"), Box::new(|a, b| a.log(b))));
        calc.ops.insert(String::from(">"),
            make_binop(String::from("Greater Than"), Box::new(|a, b| bool_to_f64(a > b))));
        calc.ops.insert(String::from("<"),
            make_binop(String::from("Less Than"), Box::new(|a, b| bool_to_f64(a < b))));
        calc.ops.insert(String::from("=="),
            make_binop(String::from("Equal?"), Box::new(|a, b| bool_to_f64(a == b))));

        // binary logical operations
        calc.ops.insert(String::from("nand"), make_binop(String::from("Nand"),
            Box::new(|a, b| bool_to_f64(
                !(f64_to_bool(a) && f64_to_bool(b))
            ))
        ));
        calc.ops.insert(String::from("and"), make_binop(String::from("And"),
            Box::new(|a, b| bool_to_f64(
                f64_to_bool(a) && f64_to_bool(b)
            ))
        ));
        calc.ops.insert(String::from("or"), make_binop(String::from("Or"),
            Box::new(|a, b| bool_to_f64(
                f64_to_bool(a) || f64_to_bool(b)
            ))
        ));
        calc.ops.insert(String::from("xor"), make_binop(String::from("Xor"),
            Box::new(|a, b| bool_to_f64(
                f64_to_bool(a) ^ f64_to_bool(b)
            ))
        ));
        
        // unary floating point operations
        calc.ops.insert(String::from("ln"),
            make_unop(String::from("Natural Logarithm"), Box::new(|x| x.ln())));
        calc.ops.insert(String::from("lg"),
            make_unop(String::from("Logarithm Base 2"), Box::new(|x| x.log2())));
        calc.ops.insert(String::from("inf?"),
            make_unop(String::from("Infinite?"), Box::new(|x| bool_to_f64(x.is_infinite()))));
        calc.ops.insert(String::from("nan?"),
            make_unop(String::from("Not A Number?"), Box::new(|x| bool_to_f64(x.is_nan()))));
        calc.ops.insert(String::from("sign"),
            make_unop(String::from("Sign"), Box::new(|x| x.signum())));
        calc.ops.insert(String::from("fin?"),
            make_unop(String::from("Finite?"), Box::new(|x| bool_to_f64(x.is_finite()))));

        // unary logical operations
        calc.ops.insert(String::from("not"), make_unop(String::from("Not"),
            Box::new(|x| bool_to_f64(!f64_to_bool(x)))));

        // stack manipulation and printing
        calc.ops.insert(String::from("print"),
            Rc::new(|mut calc| match calc.stack.pop_back() {
                None => (calc, vec![String::from("The stack is empty")]),
                Some(n) => (calc, vec![format!("{}", n)]),
            }));
        calc.ops.insert(String::from("cp"),
            Rc::new(|mut calc| match calc.stack.back().map(|n| n.clone()) {
                None => (calc, vec![String::from("'Copy' requires a non-empty stack")]),
                Some(n) => {
                    calc.stack.push_back(n);
                    (calc, vec![])
                }
            }));
        calc.ops.insert(String::from("swap"),
            Rc::new(|mut calc| {
                let len = calc.stack.len();
                if len < 2 {
                    (calc, vec![format!("'Swap' requires stack size >= 2, current = {}", len)])
                } else {
                    let a = calc.stack.pop_back().unwrap();
                    let b = calc.stack.pop_back().unwrap();
                    calc.stack.push_back(a);
                    calc.stack.push_back(b);
                    (calc, vec![])
                }
            }));

        calc
    }

    pub fn exec(self, token: String) -> (Self, Vec<String>) {
        match f64::from_str(&(*token)) {
            Ok(num) => {
                let mut new_stack = self.stack.clone();
                new_stack.push_back(num);
                (Calculator {stack: new_stack, ..self}, vec![])
            }
            Err(_) => {
                let op = self.ops.get(&token).map(|x| x.clone());
                match op {
                    None => (self, vec![format!("Unknown command '{}'", &token)]),
                    Some(func) => func(self)
                }
            }
        }
    }
}

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
