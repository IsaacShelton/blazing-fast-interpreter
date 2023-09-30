use std::io::Read;

use crate::basic_op::BasicOp;
use crate::compound_op::CompoundOp;
use crate::interpreter_op::InterpreterOp;
use std::io::Write;

const CELL_COUNT: usize = 250_000_000;

pub struct Interpreter<'ops> {
    ops: &'ops [InterpreterOp],
}

impl<'ops> Interpreter<'ops> {
    pub fn new(ops: &'ops [InterpreterOp]) -> Self {
        Self { ops }
    }

    pub fn interpret(&self) {
        let mut cells = vec![0u8; CELL_COUNT];
        let mut instr_i: usize = 0;
        let mut cell_i: usize = 0;

        while instr_i < self.ops.len() {
            match self.ops[instr_i] {
                InterpreterOp::LoopStart(distance) => {
                    if cells[cell_i] == 0 {
                        instr_i += distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::LoopEnd(distance) => {
                    if cells[cell_i] != 0 {
                        instr_i -= distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::ChangeBy(amount))) => {
                    cells[cell_i] = cells[cell_i].wrapping_add(amount);
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Shift(amount))) => {
                    cell_i = (cell_i as i64 + amount) as usize;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Input(count))) => {
                    for _ in 0..count {
                        cells[cell_i] = Self::input();
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Output(count))) => {
                    for _ in 0..count {
                        Self::output(cells[cell_i]);
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(
                    BasicOp::LoopStart | BasicOp::LoopEnd,
                )) => {
                    eprintln!("[error] Cannot execute unprocessed loop instruction");
                    return;
                }
                InterpreterOp::CompoundOp(CompoundOp::Zero) => {
                    cells[cell_i] = 0;
                    instr_i += 1;
                }
            }
        }
    }

    fn input() -> u8 {
        std::io::stdin()
            .bytes()
            .next()
            .and_then(|result| Some(result.unwrap_or_default()))
            .unwrap_or_default()
    }

    fn output(c: u8) {
        let mut stdout = std::io::stdout();
        _ = stdout.write(&[c]);
        _ = stdout.flush();
    }
}
