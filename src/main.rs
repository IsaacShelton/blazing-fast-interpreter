mod basic_op;
mod compound_op;
mod interpreter;
mod interpreter_op;

use anyhow::Result;
use basic_op::BasicOpAcc;
use compound_op::CompoundOpAcc;
use interpreter::Interpreter;
use interpreter_op::{InterpreterOp, InterpreterOpAcc};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

/*
macro_rules! connect {
    ( $last:expr ) => { $last };
    ( $head:expr, $($tail:expr), +) => {
        connect2($head, compose!($($tail),+))
    };
}

fn connect2<A, B, C, G, F>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}
*/

struct Parser {
    basic_op_acc: BasicOpAcc,
    compound_op_acc: CompoundOpAcc,
    interpreter_op_acc: InterpreterOpAcc,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            basic_op_acc: BasicOpAcc::new(),
            compound_op_acc: CompoundOpAcc::new(),
            interpreter_op_acc: InterpreterOpAcc::new(),
        }
    }

    pub fn feed(&mut self, byte: u8) -> Result<()> {
        if let Some(basic_op) = self.basic_op_acc.feed_byte(byte)? {
            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        // Flush basic op accumulator
        while let Some(basic_op) = self.basic_op_acc.finalize() {
            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        // Flush compound op accumulator
        while let Some(compound_op) = self.compound_op_acc.finalize() {
            self.interpreter_op_acc.feed(compound_op)?;
        }

        // Flush interpreter op accumulator
        // (nothing to do)

        Ok(())
    }

    pub fn view(&self) -> Result<&Vec<InterpreterOp>> {
        self.interpreter_op_acc.view()
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("[USAGE] blazing-fast-interpreter <filename.rlebf>");
        return Ok(());
    }

    let filename = &args[1];
    let mut parser = Parser::new();

    for byte in BufReader::new(File::open(filename)?).bytes() {
        parser.feed(byte?)?;
    }

    parser.flush()?;

    let interpreter = Interpreter::new(parser.view()?);

    unsafe {
        interpreter.interpret();
    }

    Ok(())
}
