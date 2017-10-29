use std::io;
use std::io::{BufRead, Write};
use std::iter::repeat;
use std::collections::{VecDeque, HashMap};
use std::str::FromStr;
use std::rc::Rc;

#[derive(Clone)]
struct OpSpec {
    pub op: Rc<Fn(Calculator) -> (Calculator, Vec<String>)>,
    pub help: String
}

#[derive(Clone)]
struct Calculator {
    stack: VecDeque<f64>,
    ops: HashMap<String, OpSpec>,
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

        let make_binop = |name: String, binop: Box<Fn(f64, f64) -> f64>, help: String|
            OpSpec{
                op: Rc::new(move |mut calc: Calculator| {
                    let len = calc.stack.len();
                    if len < 2 {
                        (calc, vec![format!("'{}' requires stack size >= 2, current = {}", name, len)])
                    } else {
                        let a = calc.stack.pop_back().unwrap();
                        let b = calc.stack.pop_back().unwrap();
                        calc.stack.push_back(binop(a, b));
                        (calc, vec![])
                    }
                }),
                help: String::from(help)
            };

        let make_unop = |name: String, unop: Box<Fn(f64) -> f64>, help: String|
            OpSpec{
                op: Rc::new(move |mut calc: Calculator| {
                    match calc.stack.pop_back() {
                        None => (calc, vec![format!("'{}' requires a non-empty stack", name)]),
                        Some(x) => {
                            calc.stack.push_back(unop(x));
                            (calc, vec![])
                        }
                    }
                }),
                help: String::from(help)
            };

        // binary floating point operations
        calc.ops.insert(String::from("-"),
            make_binop(String::from("Subtract"),
            Box::new(|a, b| a - b),
            String::from("push(pop - pop)")));
        calc.ops.insert(String::from("+"),
            make_binop(String::from("Add"),
            Box::new(|a, b| a + b),
            String::from("push(pop + pop)")));
        calc.ops.insert(String::from("*"),
            make_binop(String::from("Multiply"),
            Box::new(|a, b| a * b),
            String::from("push(pop * pop)")));
        calc.ops.insert(String::from("/"),
            make_binop(String::from("Divide"),
            Box::new(|a, b| a / b),
            String::from("push(pop / pop)")));
        calc.ops.insert(String::from("^"),
            make_binop(String::from("Exponentiate"),
            Box::new(|a, b| a.powf(b)),
            String::from("push(pop ^ pop)")));
        calc.ops.insert(String::from("*e^"),
            make_binop(String::from("Times Ten to the ..."),
            Box::new(|a, b| a * (10.0 as f64).powf(b)),
            String::from("push(pop * (10 ^ pop))")));
        calc.ops.insert(String::from("/e^"),
            make_binop(String::from("Divided by Ten to the ..."),
            Box::new(|a, b| a / (10.0 as f64).powf(b)),
            String::from("push(pop / (10 ^ pop))")));
        calc.ops.insert(String::from("log"),
            make_binop(String::from("Logarithm"),
            Box::new(|a, b| a.log(b)),
            String::from("push(log_pop(pop)")));
        calc.ops.insert(String::from(">"),
            make_binop(String::from("Greater Than"),
            Box::new(|a, b| bool_to_f64(a > b)),
            String::from("push(pop > pop)")));
        calc.ops.insert(String::from("<"),
            make_binop(String::from("Less Than"),
            Box::new(|a, b| bool_to_f64(a < b)),
            String::from("push(pop < pop)")));
        calc.ops.insert(String::from("=="),
            make_binop(String::from("Equal?"),
            Box::new(|a, b| bool_to_f64(a == b)),
            String::from("push(pop == pop)")));

        // binary logical operations
        calc.ops.insert(String::from("nand"),
            make_binop(String::from("Nand"),
            Box::new(|a, b| bool_to_f64(
                !(f64_to_bool(a) && f64_to_bool(b))
            )),
            String::from("push(not(pop and pop))")));
        calc.ops.insert(String::from("and"),
            make_binop(String::from("And"),
            Box::new(|a, b| bool_to_f64(
                f64_to_bool(a) && f64_to_bool(b)
            )),
            String::from("push(pop and pop)")));
        calc.ops.insert(String::from("or"),
            make_binop(String::from("Or"),
            Box::new(|a, b| bool_to_f64(
                f64_to_bool(a) || f64_to_bool(b)
            )),
            String::from("push(pop or pop)")));
        calc.ops.insert(String::from("xor"),
            make_binop(String::from("Xor"),
            Box::new(|a, b| bool_to_f64(
                f64_to_bool(a) ^ f64_to_bool(b)
            )),
            String::from("push(pop xor pop)")));
        
        // unary floating point operations
        calc.ops.insert(String::from("ln"),
            make_unop(String::from("Natural Logarithm"),
            Box::new(|x| x.ln()),
            String::from("push(ln(pop))")));
        calc.ops.insert(String::from("lg"),
            make_unop(String::from("Logarithm Base 2"),
            Box::new(|x| x.log2()),
            String::from("push(lg(pop))")));
        calc.ops.insert(String::from("inf?"),
            make_unop(String::from("Infinite?"),
            Box::new(|x| bool_to_f64(x.is_infinite())),
            String::from("push(whether pop is infinite)")));
        calc.ops.insert(String::from("nan?"),
            make_unop(String::from("Not A Number?"),
            Box::new(|x| bool_to_f64(x.is_nan())),
            String::from("push(whether pop is NaN)")));
        calc.ops.insert(String::from("sign"),
            make_unop(String::from("Sign"),
            Box::new(|x| x.signum()),
            String::from("push(sign of pop)")));
        calc.ops.insert(String::from("fin?"),
            make_unop(String::from("Finite?"),
            Box::new(|x| bool_to_f64(x.is_finite())),
            String::from("push(whether pip is finite)")));

        // unary logical operations
        calc.ops.insert(String::from("not"), make_unop(
            String::from("Not"),
            Box::new(|x| bool_to_f64(!f64_to_bool(x))),
            String::from("push(not(pop))")));

        // stack manipulation and printing
        calc.ops.insert(String::from("print"),
            OpSpec{
                op: Rc::new(|mut calc| match calc.stack.pop_back() {
                    None => (calc, vec![String::from("The stack is empty")]),
                    Some(n) => (calc, vec![format!("{}", n)]),
                }),
                help: String::from("print(pop)")
            }
        );
        calc.ops.insert(String::from("cp"),
            OpSpec{
                op: Rc::new(|mut calc| match calc.stack.back().map(|n| n.clone()) {
                    None => (calc, vec![String::from("'Copy' requires a non-empty stack")]),
                    Some(n) => {
                        calc.stack.push_back(n);
                        (calc, vec![])
                    }
                }),
                help: String::from("push(copy(pop))")
            }
        );
        calc.ops.insert(String::from("swap"),
            OpSpec{
                op: Rc::new(|mut calc| {
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
                }),
                help: String::from("a = pop, b = pop, push(a), push(b)")
            }
        );
        calc.ops.insert(String::from("help"),
            OpSpec{
                op: Rc::new(|calc| {
                    let msgs: Vec<String> = calc.ops.iter().map(|(k, v)| {
                        format!("{}:\n\t{}\n", k, v.help)
                    }).collect();
                    (calc, msgs)
                }),
                help: String::from("Display this help message"),
            }
        );

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
                    Some(op_spec) => (op_spec.op)(self)
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
