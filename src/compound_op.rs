use crate::basic_op::BasicOp;

#[derive(Copy, Clone, Debug)]
struct Move {}

#[derive(Copy, Clone, Debug)]
pub enum CompoundOp {
    BasicOp(BasicOp),
    Zero,
}

pub struct CompoundOpAcc {
    building: Vec<CompoundOp>,
}

impl CompoundOpAcc {
    pub fn new() -> Self {
        Self {
            building: Vec::with_capacity(4),
        }
    }

    pub fn feed(&mut self, basic_op: BasicOp) -> Option<CompoundOp> {
        self.building.push(CompoundOp::BasicOp(basic_op));

        if self.building.len() == 3 {
            match &self.building[..] {
                [CompoundOp::BasicOp(BasicOp::LoopStart), CompoundOp::BasicOp(BasicOp::ChangeBy(1 | u8::MAX)), CompoundOp::BasicOp(BasicOp::LoopEnd)] =>
                    {
                        self.building.clear();
                        Some(CompoundOp::Zero)
                    }
                _ => {
                    Some(self.building.remove(0))
                }
            }
        } else {
            None
        }
    }

    pub fn finalize(&mut self) -> Option<CompoundOp> {
        if self.building.len() > 0 {
            Some(self.building.remove(0))
        } else {
            None
        }
    }
}
