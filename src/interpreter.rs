use std::io::Read;

use crate::{basic_op::BasicOp, compound_op::CompoundOp, interpreter_op::InterpreterOp};
use std::io::Write;

pub const CELL_COUNT: usize = 25_000_000;

pub struct Interpreter<'ops> {
    ops: &'ops [InterpreterOp],
}

impl<'ops> Interpreter<'ops> {
    pub fn new(ops: &'ops [InterpreterOp]) -> Self {
        Self { ops }
    }

    #[profiling::function]
    pub unsafe fn interpret<const BOUNDS_CHECKS: bool>(&self) {
        let mut cells = vec![0u8; CELL_COUNT];
        let mut instr_i: usize = 0;
        let mut cell_i: usize = 0;

        while instr_i < self.ops.len() {
            match &self.ops[instr_i] {
                InterpreterOp::LoopStart(distance) => {
                    profiling::scope!("LoopStart");
                    if *get::<BOUNDS_CHECKS>(&cells, cell_i) == 0 {
                        instr_i += distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::LoopEnd(distance) => {
                    profiling::scope!("LoopEnd");
                    if *get::<BOUNDS_CHECKS>(&cells, cell_i) != 0 {
                        instr_i -= distance;
                    } else {
                        instr_i += 1;
                    }
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::ChangeBy(amount))) => {
                    profiling::scope!("ChangeBy");
                    let new_value = (*get::<BOUNDS_CHECKS>(&cells, cell_i)).wrapping_add(*amount);
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = new_value;
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
                        *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = Self::input();
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Output(count))) => {
                    profiling::scope!("Output");
                    for _ in 0..*count {
                        let cell_value = *get::<BOUNDS_CHECKS>(&cells, cell_i);
                        Self::output(&[cell_value]);
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::LoopStart | BasicOp::LoopEnd)) => {
                    eprintln!("[error] Cannot execute unprocessed loop instruction");
                    return;
                }
                InterpreterOp::CompoundOp(CompoundOp::Panic(value)) => {
                    eprintln!(
                        "[PANIC] Program entered panic loop with error code {}, instr_i = {}, cell_i = {}",
                        value, instr_i, cell_i
                    );
                    eprintln!("Memory before panic:");
                    let forward_range = 20;
                    let back_range = 20;
                    let start = if cell_i >= back_range { cell_i - back_range } else { 0 };
                    let end = if cell_i + forward_range < CELL_COUNT {
                        cell_i + forward_range
                    } else {
                        CELL_COUNT
                    };
                    for i in start..end {
                        eprintln!("cell {} is {}", i, cells[i]);
                    }
                    return;
                }
                InterpreterOp::CompoundOp(CompoundOp::Zero) => {
                    profiling::scope!("Zero");
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ZeroAdvance(amount)) => {
                    profiling::scope!("ZeroAdvance");
                    for _ in 0..*amount {
                        *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = 0;
                        cell_i += 1;
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ZeroRetreat(amount)) => {
                    profiling::scope!("ZeroRetreat");
                    for _ in 0..*amount {
                        *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = 0;
                        cell_i -= 1;
                    }
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::Set(value)) => {
                    profiling::scope!("Set");
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = *value;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::Dupe(offset)) => {
                    profiling::scope!("Dupe");

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) =
                        *get::<BOUNDS_CHECKS>(&cells, (cell_i as i64 + *offset) as usize);

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 1) = 0;

                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BitAnd) => {
                    // Warning: Unsound

                    // a b ? ? ? ? ? ?
                    //               ^

                    profiling::scope!("BitAnd");

                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 7);
                    let b = *get::<BOUNDS_CHECKS>(&cells, cell_i - 6);

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 7) = a & b;
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 6) = 0;
                    cell_i += 2;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::WellBehavedDivMod(shift_amount)) => {
                    profiling::scope!("WellBehavedDivMod");
                    let n = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let d = *get::<BOUNDS_CHECKS>(&cells, cell_i - 1);

                    let (n_div_d, n_mod_d) = if d == 0 { (0, 0) } else { (n / d, n % d) };

                    // Optionally check boundries (lower already checked)
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 3) = 0;

                    *get_mut::<false>(&mut cells, cell_i - 2) = 0;
                    *get_mut::<false>(&mut cells, cell_i - 1) = d.wrapping_sub(n_mod_d);
                    *get_mut::<false>(&mut cells, cell_i + 0) = n_mod_d;
                    *get_mut::<false>(&mut cells, cell_i + 1) = n_div_d;
                    *get_mut::<false>(&mut cells, cell_i + 2) = 0;

                    cell_i = (cell_i as i64 + shift_amount) as usize;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::BitNeg) => {
                    profiling::scope!("BitNeg");

                    let value = *get::<BOUNDS_CHECKS>(&cells, cell_i);

                    *get_mut::<false>(&mut cells, cell_i) = !value;
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 1) = 0;

                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::Equals) => {
                    profiling::scope!("Equals");

                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i);
                    let b = *get::<BOUNDS_CHECKS>(&cells, cell_i + 1);

                    *get_mut::<false>(&mut cells, cell_i) = (a == b) as u8;
                    *get_mut::<false>(&mut cells, cell_i + 1) = 0;

                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::NotEquals) => {
                    profiling::scope!("NotEquals");

                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i);
                    let b = *get::<BOUNDS_CHECKS>(&cells, cell_i + 1);

                    *get_mut::<false>(&mut cells, cell_i) = (a != b) as u8;
                    *get_mut::<false>(&mut cells, cell_i + 1) = 0;

                    cell_i += 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ShiftLeftLogical) => {
                    profiling::scope!("ShiftLeftLogical");

                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let b = *get::<BOUNDS_CHECKS>(&cells, cell_i - 1);

                    *get_mut::<false>(&mut cells, cell_i - 2) = if b >= 8 { 0 } else { a << b };
                    *get_mut::<false>(&mut cells, cell_i - 1) = 0;
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = 0;

                    cell_i -= 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::ShiftRightLogical) => {
                    profiling::scope!("ShiftRightLogical");

                    // Optionally check upper bound
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 3) = 0;

                    // Optionally check lower bound
                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let b = *get::<false>(&cells, cell_i - 1);

                    *get_mut::<false>(&mut cells, cell_i - 2) = if b >= 8 { 0 } else { a >> b };
                    *get_mut::<false>(&mut cells, cell_i - 1) = 0;
                    *get_mut::<false>(&mut cells, cell_i) = 0;
                    *get_mut::<false>(&mut cells, cell_i + 1) = 0;
                    *get_mut::<false>(&mut cells, cell_i + 2) = 0;

                    cell_i -= 1;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::LessThan) => {
                    profiling::scope!("LessThan");

                    // Optionally check upper bound
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 1) = 0;

                    // Optionally check lower bound
                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let b = *get::<false>(&cells, cell_i - 1);

                    *get_mut::<false>(&mut cells, cell_i - 2) = (a < b) as u8;
                    *get_mut::<false>(&mut cells, cell_i - 1) = 0;
                    *get_mut::<false>(&mut cells, cell_i + 0) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::GreaterThan) => {
                    profiling::scope!("GreaterThan");

                    // Optionally check upper bound
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 1) = 0;

                    // Optionally check lower bound
                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let b = *get::<false>(&cells, cell_i - 1);

                    *get_mut::<false>(&mut cells, cell_i - 2) = (a > b) as u8;
                    *get_mut::<false>(&mut cells, cell_i - 1) = 0;
                    *get_mut::<false>(&mut cells, cell_i + 0) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::LessThanEqual) => {
                    profiling::scope!("LessThanEqual");

                    // Check upper bound
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 1) = 0;

                    // Check lower bound
                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let b = *get::<false>(&cells, cell_i - 1);

                    *get_mut::<false>(&mut cells, cell_i - 2) = (a <= b) as u8;
                    *get_mut::<false>(&mut cells, cell_i - 1) = 0;
                    *get_mut::<false>(&mut cells, cell_i + 0) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::GreaterThanEqual) => {
                    profiling::scope!("GreaterThanEqual");

                    // Check upper bound
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i + 1) = 0;

                    // Check lower bound
                    let a = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let b = *get::<false>(&cells, cell_i - 1);

                    *get_mut::<false>(&mut cells, cell_i - 2) = (a >= b) as u8;
                    *get_mut::<false>(&mut cells, cell_i - 1) = 0;
                    *get_mut::<false>(&mut cells, cell_i + 0) = 0;

                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveAdd(offset)) => {
                    profiling::scope!("MoveAdd");
                    let current_value = *get::<BOUNDS_CHECKS>(&cells, cell_i);
                    let destination = get_mut::<BOUNDS_CHECKS>(&mut cells, (cell_i as i64 + offset) as usize);
                    *destination = (*destination).wrapping_add(current_value);
                    *get_mut::<false>(&mut cells, cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveSet(offset)) => {
                    profiling::scope!("MoveSet");
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, (cell_i as i64 + offset) as usize) =
                        *get::<BOUNDS_CHECKS>(&cells, cell_i);
                    *get_mut::<false>(&mut cells, cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveAdd2(offset1, offset2)) => {
                    profiling::scope!("MoveAdd2");
                    let current_value = *get::<BOUNDS_CHECKS>(&cells, cell_i);

                    let destination1 = get_mut::<BOUNDS_CHECKS>(&mut cells, (cell_i as i64 + offset1) as usize);
                    *destination1 = (*destination1).wrapping_add(current_value);

                    let destination2 = get_mut::<BOUNDS_CHECKS>(&mut cells, (cell_i as i64 + offset2) as usize);
                    *destination2 = (*destination2).wrapping_add(current_value);

                    *get_mut::<false>(&mut cells, cell_i) = 0;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::PrintStatic(content)) => {
                    profiling::scope!("PrintStatic");
                    Self::output(&content);
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i) = *content.last().unwrap();
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU8(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("MoveCellDynamicU8");
                    let value = *get::<BOUNDS_CHECKS>(&cells, cell_i - 2);
                    let index = *get::<BOUNDS_CHECKS>(&cells, cell_i - 1);
                    let offset = *offset as usize;
                    let final_index = cell_i - 3 - offset + index as usize;
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, final_index) = value;
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 2) = index;
                    cell_i -= 2;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU16(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("MoveCellDynamicU16");

                    let bytes = [
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 2),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 1),
                    ];

                    let value = *get::<BOUNDS_CHECKS>(&cells, cell_i - 3);
                    let index = u16::from_le_bytes(bytes);

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - (*offset) as usize + index as usize) = value;

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 3) = bytes[0];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 2) = bytes[1];
                    cell_i -= 3;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::CopyCellDynamicU8(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("CopyCellDynamicU8");
                    let offset = *offset as usize;
                    let index = *get::<BOUNDS_CHECKS>(&cells, cell_i - 1) as usize;
                    let final_index = cell_i - 1 - offset + index;
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 1) = *get::<BOUNDS_CHECKS>(&cells, final_index);
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU32(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("MoveCellDynamicU32");

                    let bytes = [
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 4),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 3),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 2),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 1),
                    ];

                    let value = *get::<BOUNDS_CHECKS>(&cells, cell_i - 5);
                    let index = u32::from_le_bytes(bytes);

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - (*offset) as usize + index as usize) = value;

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 5) = bytes[0];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 4) = bytes[1];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 3) = bytes[2];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 2) = bytes[3];
                    cell_i -= 5;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::CopyCellDynamicU32(offset)) => {
                    // Warning: Unsound

                    profiling::scope!("CopyCellDynamicU32");

                    let bytes = [
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 4),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 3),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 2),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 1),
                    ];

                    let index = u32::from_le_bytes(bytes);

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 4) =
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - *offset as usize + index as usize);

                    cell_i -= 3;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::MoveCellsStaticReverse(offset, count)) => {
                    let end_src = (cell_i as i64 + 1) as usize;
                    let start_src = (cell_i as i64 - *count as i64 + 1) as usize;
                    let end_dest = (cell_i as i64 + *offset + 1) as usize;
                    let start_dest = (end_dest as i64 - *count as i64) as usize;

                    cells.copy_within(start_src..end_src, start_dest);
                    cells[start_src..end_src].fill(0);
                    cell_i -= *count as usize;
                    instr_i += 1;
                }
                InterpreterOp::CompoundOp(CompoundOp::AddU32) => {
                    // Warning: Unsound

                    let bytes1 = [
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 8),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 7),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 6),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 5),
                    ];

                    let bytes2 = [
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 4),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 3),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 2),
                        *get::<BOUNDS_CHECKS>(&cells, cell_i - 1),
                    ];

                    let a = u32::from_le_bytes(bytes1);
                    let b = u32::from_le_bytes(bytes2);

                    let result = a.wrapping_add(b).to_le_bytes();

                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 8) = result[0];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 7) = result[1];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 6) = result[2];
                    *get_mut::<BOUNDS_CHECKS>(&mut cells, cell_i - 5) = result[3];

                    cell_i -= 5;
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

unsafe fn get<const BOUNDS_CHECKS: bool>(memory: &[u8], index: usize) -> *const u8 {
    if BOUNDS_CHECKS {
        &memory[index]
    } else {
        memory.get_unchecked(index)
    }
}

unsafe fn get_mut<const BOUNDS_CHECKS: bool>(memory: &mut [u8], index: usize) -> *mut u8 {
    if BOUNDS_CHECKS {
        &mut memory[index]
    } else {
        memory.get_unchecked_mut(index)
    }
}
