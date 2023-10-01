use crate::basic_op::BasicOp;
use slice_deque::SliceDeque;

#[derive(Copy, Clone, Debug)]
struct Move {}

#[derive(Copy, Clone, Debug)]
pub enum CompoundOp {
    BasicOp(BasicOp),
    Zero,
    MoveAdd(i64),
    MoveSet(i64),
    MoveAdd2(i64, i64),
    WellBehavedDivMod(i64, usize),
}

pub struct CompoundOpAcc {
    building: SliceDeque<CompoundOp>,
}

use BasicOp::*;
use CompoundOp::*;

const WINDOW_SIZE: usize = 128;

impl CompoundOpAcc {
    pub fn new() -> Self {
        Self {
            building: SliceDeque::new(),
        }
    }

    pub fn feed(&mut self, basic_op: BasicOp) -> Option<CompoundOp> {
        self.building.push_back(CompoundOp::BasicOp(basic_op));

        match &self.building[..] {
            // Zero cell pattern
            [.., BasicOp(LoopStart), BasicOp(ChangeBy(1 | u8::MAX)), BasicOp(LoopEnd)] => {
                self.building.truncate_back(self.building.len() - 3);

                if let Some(Zero) = self.building.back() {
                    // Don't add as it's redundant
                } else {
                    self.building.push_back(Zero);
                }
            }
            // Move add 2 algorithm
            [
                ..,
                BasicOp(LoopStart),
                BasicOp(Shift(toward_amount1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(toward_amount2)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(back_amount)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
            ] if *toward_amount1 + *toward_amount2 == -*back_amount => {
                let offset1 = *toward_amount1;
                let offset2 = *toward_amount2;
                self.building.truncate_back(self.building.len() - 8);
                self.building.push_back(MoveAdd2(offset1, offset1 + offset2));
            }
            // Move add algorithm
            [
                ..,
                BasicOp(LoopStart),
                BasicOp(Shift(toward_amount)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(back_amount)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
            ] if *toward_amount == -*back_amount => {
                let offset = *toward_amount;
                self.building.truncate_back(self.building.len() - 6);
                self.building.push_back(MoveAdd(offset));

                match &self.building[..] {
                    // Move set algorithm
                    [
                        ..,
                        BasicOp(Shift(toward_amount)),
                        CompoundOp::Zero,
                        BasicOp(Shift(back_amount)),
                        CompoundOp::MoveAdd(move_add_toward_amount),
                    ] if *toward_amount == -*back_amount && *toward_amount == *move_add_toward_amount => {
                        let offset = *toward_amount;
                        self.building.truncate_back(self.building.len() - 4);
                        self.building.push_back(MoveSet(offset));
                    }
                    _ => {}
                }
            }
            // Divmod algorithm
            [
                ..,
                Zero,
                BasicOp(Shift(1)),
                Zero,
                BasicOp(Shift(1)),
                Zero,
                BasicOp(Shift(1)),
                Zero,
                BasicOp(Shift(-5)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(1)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(2)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                BasicOp(LoopStart),
                BasicOp(Shift(-2)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(2)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(1)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(2)),
                BasicOp(LoopEnd),
                BasicOp(Shift(-5)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                BasicOp(LoopStart),
                BasicOp(Shift(3)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                BasicOp(LoopStart),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(1)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(2)),
                BasicOp(LoopEnd),
                BasicOp(Shift(neg_5_plus_extra)),
            ] => {
                let default_shift_amount = -2;
                let shift_amount = neg_5_plus_extra + 5 + default_shift_amount;
                self.building.truncate_back(self.building.len() - 51);
                self.building.push_back(WellBehavedDivMod(shift_amount, 4));
            }
            _ => (),
        }

        if self.building.len() > WINDOW_SIZE {
            self.building.pop_front()
        } else {
            None
        }
    }

    pub fn finalize(&mut self) -> Option<CompoundOp> {
        self.building.pop_front()
    }
}
