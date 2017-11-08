use std::collections::{VecDeque, HashMap};
use std::str::FromStr;
use std::rc::Rc;

/// The description of a calculator operation and how to execute it.
#[derive(Clone)]
pub struct OpSpec {
    /// A counted reference to the function itself.
    /// Takes a calculator's current state.
    /// returns a list of messages to print, and the updated calculator state.
    pub op: Rc<Fn(Calculator) -> (Calculator, Vec<String>)>,
    /// A help string displayed for this operation when the user asks for help.
    pub help: String
}

/// Stores the state of a calculator at any given time.
#[derive(Clone)]
pub struct Calculator {
    /// A stack to place values in.
    pub stack: VecDeque<f64>,
    /// The operations this calculator supports.
    /// The key in this map specifies a string the user writes to invoke the operation.
    /// An operation's key string should not have any spaces in it.
    pub ops: HashMap<String, OpSpec>,
}

/// Used to encode a boolean as f64 (true -> 1.0, false -> 0.0).
fn bool_to_f64(b: bool) -> f64 {
    if b {1.0} else {0.0}
}

/// Used to decode a boolean previously stored as f64 (0.0 -> false, != 0.0 -> true).
fn f64_to_bool(f: f64) -> bool {
    f != 0.0
}

impl Calculator {

    /// Constructs a new calculator with a basic set of operations available.
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
            String::from("push(log base pop of pop")));
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
        calc.ops.insert(String::from("neg"),
            make_unop(String::from("Negate"),
            Box::new(|x| -x),
            String::from("push(whether pip is finite)")));
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

    /// Executes a single token on the calculator, returning its new state and some messages.
    /// This implementation simply selects an operation based on the provided token
    ///   and returns the result of executing that operation on the calculator's current state.
    pub fn exec(mut self, token: String) -> (Self, Vec<String>) {
        match f64::from_str(&(*token)) {
            Ok(num) => {
                self.stack.push_back(num);
                (self, vec![])
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