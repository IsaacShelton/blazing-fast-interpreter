mod basic_op;
mod compound_op;
mod interpreter;
mod interpreter_op;

use anyhow::Result;
use basic_op::BasicOpAcc;
use clap::{command, Arg};
use compound_op::{CompoundOpAcc, CompoundOp};
use interpreter::Interpreter;
use interpreter_op::{InterpreterOp, InterpreterOpAcc};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

struct Parser {
    basic_op_acc: BasicOpAcc,
    compound_op_acc: CompoundOpAcc,
    interpreter_op_acc: InterpreterOpAcc,

    emit_ops_file: Option<File>,
}

impl Parser {
    pub fn new(emit_ops_filename: Option<&str>) -> Result<Self> {
        let emit_ops_file = if let Some(filename) = emit_ops_filename {
            Some(File::create(filename)?)
        } else {
            None
        };

        Ok(Self {
            basic_op_acc: BasicOpAcc::new(),
            compound_op_acc: CompoundOpAcc::new(),
            interpreter_op_acc: InterpreterOpAcc::new(),
            emit_ops_file,
        })
    }

    pub fn feed(&mut self, byte: u8) -> Result<()> {
        if let Some(basic_op) = self.basic_op_acc.feed_byte(byte)? {
            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.log(&compound_op)?;
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        // Flush basic op accumulator
        while let Some(basic_op) = self.basic_op_acc.finalize() {
            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.log(&compound_op)?;
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        // Flush compound op accumulator
        while let Some(compound_op) = self.compound_op_acc.finalize() {
            self.log(&compound_op)?;
            self.interpreter_op_acc.feed(compound_op)?;
        }

        // Flush interpreter op accumulator
        // (nothing to do)

        Ok(())
    }

    fn log(&mut self, compound_op: &CompoundOp) -> Result<()> {
        // Write to output if requested
        if let Some(emit_ops_file) = &mut self.emit_ops_file {
            writeln!(emit_ops_file, "{:?}", compound_op)?;
        }

        Ok(())
    }

    pub fn view(&self) -> Result<&Vec<InterpreterOp>> {
        self.interpreter_op_acc.view()
    }
}

fn main() -> Result<()> {
    // Starting the Tracy client is necessary before any invoking any of its APIs
    #[cfg(feature = "profile")]
    tracy_client::Client::start();

    // Good to call this on any threads that are created to get clearer profiling results
    profiling::register_thread!("Main Thread");

    let args = command!()
        .about("A blazing fast interpreter for running BrainF*ck programs")
        .arg(Arg::new("filename").required(true))
        .arg(Arg::new("emit-ops").long("emit-ops").value_name("FILE"))
        .get_matches();

    let filename = args.get_one::<String>("filename").unwrap();
    let mut parser = Parser::new(args.get_one::<String>("emit-ops").map(|x| x.as_str()))?;

    for byte in BufReader::new(File::open(filename)?).bytes() {
        parser.feed(byte?)?;
    }

    parser.flush()?;

    let interpreter = Interpreter::new(parser.view()?);

    if !args.contains_id("emit-ops") {
        unsafe {
            interpreter.interpret();
        }
    }

    Ok(())
}
