mod basic_op;
mod compound_op;
mod interpreter;
mod interpreter_op;
mod transpile_c;

use anyhow::Result;
use basic_op::{BasicOpAcc, BasicOp};
use clap::{command, Arg, ArgAction};
use compound_op::{CompoundOp, CompoundOpAcc};
use interpreter::Interpreter;
use interpreter_op::{InterpreterOp, InterpreterOpAcc};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
use transpile_c::transpile_c;

struct Parser {
    basic_op_acc: BasicOpAcc,
    compound_op_acc: CompoundOpAcc,
    interpreter_op_acc: InterpreterOpAcc,

    emit_simplified_filename: Option<File>,
    emit_ops_file: Option<File>,
}

impl Parser {
    pub fn new(emit_simplified_filename: Option<&str>, emit_ops_filename: Option<&str>) -> Result<Self> {
        let emit_simplified_filename = emit_simplified_filename.and_then(|filename| Some(File::create(filename))).transpose()?;
        let emit_ops_file = emit_ops_filename.and_then(|filename| Some(File::create(filename))).transpose()?;

        Ok(Self {
            basic_op_acc: BasicOpAcc::new(),
            compound_op_acc: CompoundOpAcc::new(),
            interpreter_op_acc: InterpreterOpAcc::new(),
            emit_simplified_filename,
            emit_ops_file,
        })
    }

    pub fn feed(&mut self, byte: u8) -> Result<()> {
        if let Some(basic_op) = self.basic_op_acc.feed_byte(byte)? {
            self.log_simplified_op(&basic_op)?;

            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.log_compound_op(&compound_op)?;
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        while let Some(basic_op) = self.basic_op_acc.continued() {
            self.log_simplified_op(&basic_op)?;

            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.log_compound_op(&compound_op)?;
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        // Flush basic op accumulator
        while let Some(basic_op) = self.basic_op_acc.finalize() {
            self.log_simplified_op(&basic_op)?;

            if let Some(compound_op) = self.compound_op_acc.feed(basic_op) {
                self.log_compound_op(&compound_op)?;
                self.interpreter_op_acc.feed(compound_op)?;
            }
        }

        // Flush compound op accumulator
        while let Some(compound_op) = self.compound_op_acc.finalize() {
            self.log_compound_op(&compound_op)?;
            self.interpreter_op_acc.feed(compound_op)?;
        }

        // Flush interpreter op accumulator
        // (nothing to do)

        Ok(())
    }

    fn log_simplified_op(&mut self, basic_op: &BasicOp) -> Result<()> {
        // Write to output if requested
        if let Some(emit_simplified_filename) = &mut self.emit_simplified_filename {
            write!(emit_simplified_filename, "{}", basic_op)?;
        }

        Ok(())
    }

    fn log_compound_op(&mut self, compound_op: &CompoundOp) -> Result<()> {
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
        .arg(Arg::new("emit-simplified").long("emit-simplified").value_name("FILE"))
        .arg(
            Arg::new("bounds-checks")
                .long("bounds-checks")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("transpile-c").long("transpile-c").value_name("OUT_FILE"))
        .get_matches();

    let filename = args.get_one::<String>("filename").unwrap();
    let emit_simplified_filename = args.get_one::<String>("emit-simplified").map(|x| x.as_str());
    let emit_ops_filename = args.get_one::<String>("emit-ops").map(|x| x.as_str());
    let mut parser = Parser::new(emit_simplified_filename, emit_ops_filename)?;

    for byte in BufReader::new(File::open(filename)?).bytes() {
        parser.feed(byte?)?;
    }

    parser.flush()?;

    if args.contains_id("transpile-c") {
        return transpile_c(parser.view()?.iter(), args.get_one::<String>("transpile-c").unwrap());
    }

    let interpreter = Interpreter::new(parser.view()?);

    if !args.contains_id("emit-ops") && !args.contains_id("emit-simplified") {
        if args.get_flag("bounds-checks") {
            unsafe {
                interpreter.interpret::<true>();
            }
        } else {
            unsafe {
                interpreter.interpret::<false>();
            }
        }
    }

    Ok(())
}
