use std::env;
use std::fs;
use std::io::{self, prelude::*};
use std::process;

enum Op {
    Dec,
    Inc,
    Prev,
    Next,
    Loop(Box<[Op]>),
    Print,
}
use Op::*;

struct Tape {
    pos: usize,
    tape: Vec<i32>,
}

impl Tape {
    fn new() -> Self {
        Default::default()
    }

    fn get(&self) -> i32 {
        // Always safe
        unsafe { *self.tape.get_unchecked(self.pos) }
    }

    fn dec(&mut self) {
        // Always safe
        unsafe {
            *self.tape.get_unchecked_mut(self.pos) -= 1;
        }
    }

    fn inc(&mut self) {
        // Always safe
        unsafe {
            *self.tape.get_unchecked_mut(self.pos) += 1;
        }
    }

    fn prev(&mut self) {
        self.pos -= 1;
    }

    fn next(&mut self) {
        self.pos += 1;
        if self.pos >= self.tape.len() {
            self.tape.resize(self.pos << 1, 0);
        }
    }
}

impl Default for Tape {
    fn default() -> Self {
        Self {
            pos: 0,
            tape: vec![0],
        }
    }
}

#[derive(Default)]
struct Printer {
    sum1: i32,
    sum2: i32,
    quiet: bool,
}

impl Printer {
    fn new(quiet: bool) -> Self {
        Self {
            quiet,
            ..Default::default()
        }
    }

    fn print(&mut self, n: i32) {
        if self.quiet {
            self.sum1 = (self.sum1 + n) % 255;
            self.sum2 = (self.sum2 + self.sum1) % 255;
        } else {
            let mut stdout = io::stdout();
            stdout.lock().write_all(&[n as u8]).ok();
            stdout.flush().ok();
        }
    }

    fn get_checksum(&self) -> i32 {
        (self.sum2 << 8) | self.sum1
    }
}

fn run(program: &[Op], tape: &mut Tape, p: &mut Printer) {
    for op in program {
        match *op {
            Dec => tape.dec(),
            Inc => tape.inc(),
            Prev => tape.prev(),
            Next => tape.next(),
            Loop(ref program) => {
                while tape.get() > 0 {
                    run(program, tape, p);
                }
            }
            Print => {
                p.print(tape.get());
            }
        }
    }
}

fn parse<I: Iterator<Item = char>>(it: &mut I) -> Box<[Op]> {
    let mut buf = vec![];
    while let Some(c) = it.next() {
        buf.push(match c {
            '-' => Dec,
            '+' => Inc,
            '<' => Prev,
            '>' => Next,
            '.' => Print,
            '[' => Loop(parse(it)),
            ']' => break,
            _ => continue,
        });
    }
    buf.into_boxed_slice()
}

struct Program {
    ops: Box<[Op]>,
}

impl Program {
    fn new(code: &str) -> Self {
        Self {
            ops: parse(&mut code.chars()),
        }
    }

    fn run(&self, p: &mut Printer) {
        let mut tape = Tape::new();
        run(&self.ops, &mut tape, p);
    }
}

fn notify(msg: &str) {
    if let Ok(mut stream) = std::net::TcpStream::connect("localhost:9001") {
        stream.write_all(msg.as_bytes()).ok();
    }
}

fn verify() {
    let s = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>
             ---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    let mut p_left = Printer::new(true);
    Program::new(s).run(&mut p_left);
    let left = p_left.get_checksum();

    let mut p_right = Printer::new(true);
    for c in "Hello World!\n".chars() {
        p_right.print(c as i32);
    }
    let right = p_right.get_checksum();
    if left != right {
        eprintln!("{} != {}", left, right);
        process::exit(-1);
    }
}

fn main() {
    verify();

    let arg1 = env::args().nth(1).unwrap();
    let s = fs::read_to_string(arg1).unwrap();
    let mut p = Printer::new(env::var("QUIET").is_ok());

    notify(&format!("Rust\t{}", process::id()));
    Program::new(&s).run(&mut p);
    notify("stop");

    if p.quiet {
        println!("Output checksum: {}", p.get_checksum());
    }
}

