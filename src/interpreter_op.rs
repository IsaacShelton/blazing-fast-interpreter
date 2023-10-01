use crate::{basic_op::BasicOp, compound_op::CompoundOp};
use anyhow::{anyhow, Result};

#[derive(Copy, Clone, Debug)]
pub enum InterpreterOp {
    CompoundOp(CompoundOp),
    LoopStart(usize),
    LoopEnd(usize),
}

pub struct InterpreterOpAcc {
    ops: Vec<InterpreterOp>,
    loop_start_indices: Vec<usize>,
}

impl InterpreterOpAcc {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            loop_start_indices: Vec::new(),
        }
    }

    pub fn feed(&mut self, compound_op: CompoundOp) -> Result<()> {
        match compound_op {
            CompoundOp::BasicOp(BasicOp::LoopStart) => {
                self.loop_start_indices.push(self.ops.len());
                self.ops.push(InterpreterOp::LoopStart(0));
            }
            CompoundOp::BasicOp(BasicOp::LoopEnd) => match self.loop_start_indices.pop() {
                Some(start_index) => {
                    let end_index = self.ops.len();
                    let distance = end_index - start_index;
                    self.ops.push(InterpreterOp::LoopEnd(distance));
                    self.ops[start_index] = InterpreterOp::LoopStart(distance);
                }
                None => {
                    return Err(anyhow!("Instruction ']' is missing match"));
                }
            },
            _ => self.ops.push(InterpreterOp::CompoundOp(compound_op)),
        }

        Ok(())
    }

    pub fn view(&self) -> Result<&Vec<InterpreterOp>> {
        if self.loop_start_indices.len() != 0 {
            return Err(anyhow!("Instruction '[' is missing match"));
        }

        Ok(&self.ops)
    }
}
