use std::{fs::read_to_string, io::Write};

struct Tape {
    pos: usize,
    tape: Vec<i32>,
}

impl Tape {
    fn new() -> Self {
        Self {
            pos: 0,
            tape: vec![0],
        }
    }

    fn get(&self) -> i32 {
        unsafe { *self.tape.get_unchecked(self.pos) }
    }

    fn dec(&mut self) {
        unsafe {
            *self.tape.get_unchecked_mut(self.pos) -= 1;
        }
    }

    fn inc(&mut self) {
        unsafe {
            *self.tape.get_unchecked_mut(self.pos) += 1;
        }
    }

    fn prev(&mut self) {
        debug_assert_ne!(self.pos, 0);
        self.pos -= 1;
    }

    fn next(&mut self) {
        self.pos += 1;
        if self.pos >= self.tape.len() {
            self.tape.resize(self.pos * 2, 0);
        }
    }
}

fn run(mut program: std::str::Chars) {
    let mut tape = Tape::new();
    let mut loop_stack = Vec::new();
    let mut stdout = std::io::stdout();

    while let Some(op) = program.next() {
        match op {
            '-' => tape.dec(),
            '+' => tape.inc(),
            '<' => tape.prev(),
            '>' => tape.next(),
            '[' => {
                if tape.get() > 0 {
                    loop_stack.push(program.clone());
                } else {
                    let mut loop_stack_size_delta = 1;

                    while let Some(op) = program.next() {
                        match op {
                            '[' => loop_stack_size_delta += 1,
                            ']' => {
                                loop_stack_size_delta -= 1;

                                if loop_stack_size_delta == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ']' => {
                if tape.get() > 0 {
                    program = loop_stack.last().expect("Missing [").clone();
                } else {
                    loop_stack.pop();
                }
            }
            '.' => {
                let _ = stdout
                    .write_all(&[tape.get() as u8])
                    .and_then(|_| stdout.flush());
            }
            _ => {}
        }
    }
}

fn notify(msg: &str) {
    if let Ok(mut stream) = std::net::TcpStream::connect("localhost:9001") {
        stream.write_all(msg.as_bytes()).ok();
    }
}

fn main() {
    let code = std::env::args()
        .nth(1)
        .and_then(|file_path| read_to_string(file_path).ok())
        .expect("Expected a valid file path!");

    notify(&format!("Rust\t{}", std::process::id()));
    run(code.chars());
    notify("stop");
}
