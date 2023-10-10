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

    #[profiling::function]
    pub unsafe fn interpret(&self) {
        let mut cells = vec![0u8; CELL_COUNT];
        let mut instr_i: usize = 0;
        let mut cell_i: usize = 0;

        while instr_i < self.ops.len() {
            match &self.ops[instr_i] {
                InterpreterOp::LoopStart(distance) => {
                    profiling::scope!("LoopStart");
                    if *cells.get_unchecked(cell_i) == 0 {
                        instr_i += distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::LoopEnd(distance) => {
                    profiling::scope!("LoopEnd");
                    if *cells.get_unchecked(cell_i) != 0 {
                        instr_i -= distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::ChangeBy(amount))) => {
                    profiling::scope!("ChangeBy");
                    *cells.get_unchecked_mut(cell_i) = cells.get_unchecked(cell_i).wrapping_add(*amount);
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Shift(amount))) => {
                    profiling::scope!("Shift");
                    cell_i = (cell_i as i64 + amount) as usize;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Input(count))) => {
                    profiling::scope!("Input");
                    for _ in 0..*count {
                        cells[cell_i] = Self::input();
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Output(count))) => {
                    profiling::scope!("Output");
                    for _ in 0..*count {
                        Self::output(&[cells[cell_i]]);
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::LoopStart | BasicOp::LoopEnd)) => {
                    eprintln!("[error] Cannot execute unprocessed loop instruction");
                    return;
                }
                InterpreterOp::CompoundOp(CompoundOp::Panic(value)) => {
                    eprintln!("[PANIC] Program entered panic loop with error code {}, instr_i = {}, cell_i = {}", value, instr_i, cell_i);
                    eprintln!("Memory before panic:");
                    let range = 20;
                    let start = if cell_i >= range { cell_i - range } else { 0 };
                    let end = if cell_i + range < CELL_COUNT { cell_i + range } else { CELL_COUNT };
                    for i in start..end {
                        eprintln!("cell {} is {}", i, *cells.get_unchecked_mut(i));
                    }
                    return;
                }
                InterpreterOp::CompoundOp(CompoundOp::Zero) => {
                    profiling::scope!("Zero");
                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ZeroAdvance(amount)) => {
                    profiling::scope!("ZeroAdvance");
                    for _ in 0..*amount {
                        *cells.get_unchecked_mut(cell_i) = 0;
                        cell_i += 1;
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ZeroRetreat(amount)) => {
                    profiling::scope!("ZeroRetreat");
                    for _ in 0..*amount {
                        *cells.get_unchecked_mut(cell_i) = 0;
                        cell_i -= 1;
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::Set(value)) => {
                    profiling::scope!("Set");
                    *cells.get_unchecked_mut(cell_i) = *value;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::Dupe(offset)) => {
                    profiling::scope!("Dupe");
                    *cells.get_unchecked_mut(cell_i) = *cells.get_unchecked((cell_i as i64 + *offset) as usize);
                    *cells.get_unchecked_mut(cell_i + 1) = 0;
                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BitAnd) => {
                    // Warning: Unsound

                    // a b ? ? ? ? ? ?
                    //               ^

                    profiling::scope!("BitAnd");

                    let a = *cells.get_unchecked(cell_i - 7);
                    let b = *cells.get_unchecked(cell_i - 6);

                    *cells.get_unchecked_mut(cell_i - 7) = a & b;
                    *cells.get_unchecked_mut(cell_i - 6) = 0;
                    cell_i += 2;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::WellBehavedDivMod(shift_amount)) => {
                    profiling::scope!("WellBehavedDivMod");
                    let n = *cells.get_unchecked(cell_i - 2);
                    let d = *cells.get_unchecked(cell_i - 1);

                    let (n_div_d, n_mod_d) = if d == 0 { (0, 0) } else { (n / d, n % d) };

                    *cells.get_unchecked_mut(cell_i - 2) = 0;
                    *cells.get_unchecked_mut(cell_i - 1) = d.wrapping_sub(n_mod_d);
                    *cells.get_unchecked_mut(cell_i) = n_mod_d;
                    *cells.get_unchecked_mut(cell_i + 1) = n_div_d;
                    *cells.get_unchecked_mut(cell_i + 2) = 0;
                    *cells.get_unchecked_mut(cell_i + 3) = 0;

                    cell_i = (cell_i as i64 + shift_amount) as usize;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BitNeg) => {
                    profiling::scope!("BitNeg");

                    let a = *cells.get_unchecked(cell_i);
                    *cells.get_unchecked_mut(cell_i) = !a;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;
                    
                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::Equals) => {
                    profiling::scope!("Equals");

                    let a = *cells.get_unchecked(cell_i);
                    let b = *cells.get_unchecked(cell_i + 1);
                    *cells.get_unchecked_mut(cell_i) = (a == b) as u8;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;

                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::NotEquals) => {
                    profiling::scope!("NotEquals");

                    let a = *cells.get_unchecked(cell_i);
                    let b = *cells.get_unchecked(cell_i + 1);

                    *cells.get_unchecked_mut(cell_i) = (a != b) as u8;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;

                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ShiftLeftLogical) => {
                    profiling::scope!("ShiftLeftLogical");

                    let a = *cells.get_unchecked(cell_i - 2);
                    let b = *cells.get_unchecked(cell_i - 1);

                    *cells.get_unchecked_mut(cell_i - 2) = if b >= 8 { 0 } else { a << b };
                    *cells.get_unchecked_mut(cell_i - 1) = 0;
                    *cells.get_unchecked_mut(cell_i) = 0;

                    cell_i -= 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ShiftRightLogical) => {
                    profiling::scope!("ShiftRightLogical");
                    let a = *cells.get_unchecked(cell_i - 2);
                    let b = *cells.get_unchecked(cell_i - 1);

                    *cells.get_unchecked_mut(cell_i - 2) = if b >= 8 { 0 } else { a >> b };
                    *cells.get_unchecked_mut(cell_i - 1) = 0;
                    *cells.get_unchecked_mut(cell_i) = 0;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;
                    *cells.get_unchecked_mut(cell_i + 2) = 0;
                    *cells.get_unchecked_mut(cell_i + 3) = 0;

                    cell_i -= 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::LessThan) => {
                    profiling::scope!("LessThan");
                    let a = *cells.get_unchecked(cell_i - 2);
                    let b = *cells.get_unchecked(cell_i - 1);

                    *cells.get_unchecked_mut(cell_i - 2) = (a < b) as u8;
                    *cells.get_unchecked_mut(cell_i - 1) = 0;
                    *cells.get_unchecked_mut(cell_i + 0) = 0;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::GreaterThan) => {
                    profiling::scope!("GreaterThan");
                    let a = *cells.get_unchecked(cell_i - 2);
                    let b = *cells.get_unchecked(cell_i - 1);

                    *cells.get_unchecked_mut(cell_i - 2) = (a > b) as u8;
                    *cells.get_unchecked_mut(cell_i - 1) = 0;
                    *cells.get_unchecked_mut(cell_i + 0) = 0;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::LessThanEqual) => {
                    profiling::scope!("LessThanEqual");
                    let a = *cells.get_unchecked(cell_i - 2);
                    let b = *cells.get_unchecked(cell_i - 1);

                    *cells.get_unchecked_mut(cell_i - 2) = (a <= b) as u8;
                    *cells.get_unchecked_mut(cell_i - 1) = 0;
                    *cells.get_unchecked_mut(cell_i + 0) = 0;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::GreaterThanEqual) => {
                    profiling::scope!("GreaterThanEqual");
                    let a = *cells.get_unchecked(cell_i - 2);
                    let b = *cells.get_unchecked(cell_i - 1);

                    *cells.get_unchecked_mut(cell_i - 2) = (a >= b) as u8;
                    *cells.get_unchecked_mut(cell_i - 1) = 0;
                    *cells.get_unchecked_mut(cell_i + 0) = 0;
                    *cells.get_unchecked_mut(cell_i + 1) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveAdd(offset)) => {
                    profiling::scope!("MoveAdd");
                    let current_value = *cells.get_unchecked(cell_i);
                    let destination = cells.get_unchecked_mut((cell_i as i64 + offset) as usize);
                    *destination = destination.wrapping_add(current_value);
                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveSet(offset)) => {
                    profiling::scope!("MoveSet");
                    *cells.get_unchecked_mut((cell_i as i64 + offset) as usize) = *cells.get_unchecked(cell_i);
                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveAdd2(offset1, offset2)) => {
                    profiling::scope!("MoveAdd2");
                    let current_value = *cells.get_unchecked(cell_i);

                    let destination1 = cells.get_unchecked_mut((cell_i as i64 + offset1) as usize);
                    *destination1 = destination1.wrapping_add(current_value);

                    let destination2 = cells.get_unchecked_mut((cell_i as i64 + offset2) as usize);
                    *destination2 = destination2.wrapping_add(current_value);

                    *cells.get_unchecked_mut(cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::PrintStatic(content)) => {
                    profiling::scope!("PrintStatic");
                    Self::output(&content);
                    *cells.get_unchecked_mut(cell_i) = *content.last().unwrap();
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU8(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("MoveCellDynamicU8");
                    let value = *cells.get_unchecked(cell_i - 2);
                    let index = *cells.get_unchecked_mut(cell_i - 1) as usize;
                    let offset = *offset as usize;
                    let final_index = cell_i - 3 - offset + index;
                    *cells.get_unchecked_mut(final_index) = value;
                    cell_i -= 2;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::CopyCellDynamicU8(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("CopyCellDynamicU8");
                    let offset = *offset as usize;
                    let index = *cells.get_unchecked_mut(cell_i) as usize;
                    let final_index = cell_i - 1 - offset + index;
                    *cells.get_unchecked_mut(cell_i - 1) = *cells.get_unchecked(final_index);
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

    fn output(slice: &[u8]) {
        let mut stdout = std::io::stdout();
        _ = stdout.write(slice);
        _ = stdout.flush();
    }
}
