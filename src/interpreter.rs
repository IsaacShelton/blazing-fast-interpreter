use std::io::Read;

use crate::{basic_op::BasicOp, compound_op::CompoundOp, interpreter_op::InterpreterOp};
use std::io::Write;

const CELL_COUNT: usize = 250_000_000;

pub struct Interpreter<'ops> {
    ops: &'ops [InterpreterOp],
}

impl<'ops> Interpreter<'ops> {
    pub fn new(ops: &'ops [InterpreterOp]) -> Self {
        Self { ops }
    }

    pub unsafe fn interpret(&self) {
        let mut cells = vec![0u8; CELL_COUNT];
        let mut instr_i: usize = 0;
        let mut cell_i: usize = 0;

        while instr_i < self.ops.len() {
            // println!("{:?}", self.ops[instr_i]);

            match self.ops[instr_i] {
                InterpreterOp::LoopStart(distance) => {
                    if *cells.get_unchecked(cell_i) == 0 {
                        instr_i += distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::LoopEnd(distance) => {
                    if *cells.get_unchecked(cell_i) != 0 {
                        instr_i -= distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::ChangeBy(amount))) => {
                    *cells.get_unchecked_mut(cell_i) = cells.get_unchecked(cell_i).wrapping_add(amount);
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
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::LoopStart | BasicOp::LoopEnd)) => {
                    eprintln!("[error] Cannot execute unprocessed loop instruction");
                    return;
                }
                InterpreterOp::CompoundOp(CompoundOp::Zero) => {
                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::WellBehavedDivMod(shift_amount, cells_to_zero)) => {
                    for i in 2..(cells_to_zero + 2) {
                        cells[cell_i + i] = 0;
                    }

                    let n = *cells.get_unchecked(cell_i - 2);
                    let d = *cells.get_unchecked(cell_i - 1);

                    let n_div_d = n.checked_div(d).unwrap_or(0);
                    let n_mod_d = n.checked_rem(d).unwrap_or(0);

                    *cells.get_unchecked_mut(cell_i - 2) = 0;
                    *cells.get_unchecked_mut(cell_i - 1) = d.wrapping_sub(n_mod_d);
                    *cells.get_unchecked_mut(cell_i) = n_mod_d;
                    *cells.get_unchecked_mut(cell_i + 1) = n_div_d;

                    cell_i = (cell_i as i64 + shift_amount) as usize;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveAdd(offset)) => {
                    let current_value = *cells.get_unchecked(cell_i);
                    let destination = cells.get_unchecked_mut((cell_i as i64 + offset) as usize);
                    *destination = destination.wrapping_add(current_value);
                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveSet(offset)) => {
                    *cells.get_unchecked_mut((cell_i as i64 + offset) as usize) = *cells.get_unchecked(cell_i);
                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveAdd2(offset1, offset2)) => {
                    let current_value = *cells.get_unchecked(cell_i);

                    let destination1 = cells.get_unchecked_mut((cell_i as i64 + offset1) as usize);
                    *destination1 = destination1.wrapping_add(current_value);

                    let destination2 = cells.get_unchecked_mut((cell_i as i64 + offset2) as usize);
                    *destination2 = destination2.wrapping_add(current_value);

                    *cells.get_unchecked_mut(cell_i) = 0;
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
