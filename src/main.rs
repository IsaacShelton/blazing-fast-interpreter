mod basic_op;
mod compound_op;
mod interpreter;
mod interpreter_op;

use anyhow::Result;
use basic_op::BasicOpAcc;
use compound_op::CompoundOpAcc;
use std::io::prelude::*;
use std::{fs::File, io::BufReader};
use interpreter_op::InterpreterOpAcc;
use interpreter::Interpreter;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("[USAGE] blazing-fast-interpreter <filename.rlebf>");
        return Ok(());
    }

    let filename = &args[1];
    let mut basic_op_acc = BasicOpAcc::new();
    let mut compound_op_acc = CompoundOpAcc::new();
    let mut interpreter_op_acc = InterpreterOpAcc::new();

    for byte in BufReader::new(File::open(filename)?).bytes() {
        if let Some(basic_op) = basic_op_acc.feed_byte(byte?)? {
            if let Some(compound_op) = compound_op_acc.feed(basic_op) {
                interpreter_op_acc.feed(compound_op)?;
            }
        }
    }

    while let Some(basic_op) = basic_op_acc.finalize() {
        if let Some(compound_op) = compound_op_acc.feed(basic_op) {
            interpreter_op_acc.feed(compound_op)?;
        }
    }

    while let Some(compound_op) = compound_op_acc.finalize() {
        interpreter_op_acc.feed(compound_op)?;
    }

    let interpreter = Interpreter::new(interpreter_op_acc.view()?);
    interpreter.interpret();
    Ok(())
}

