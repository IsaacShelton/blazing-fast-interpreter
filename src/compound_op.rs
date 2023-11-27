use crate::basic_op::BasicOp;
use slice_deque::SliceDeque;

#[derive(Clone, Debug)]
pub enum CompoundOp {
    BasicOp(BasicOp),
    Panic(u8),
    Zero,
    ZeroAdvance(u64),
    ZeroRetreat(u64),
    Set(u8),
    MoveAdd(i64),
    MoveAdd2(i64, i64),
    MoveSet(i64),
    Dupe(i64),
    Equals,
    NotEquals,
    ShiftLeftLogical,
    ShiftRightLogical,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    BitAnd,
    BitNeg,
    WellBehavedDivMod(i64),
    PrintStatic(Vec<u8>),
    MoveCellDynamicU8(u64),
    CopyCellDynamicU8(u64),
}

pub struct CompoundOpAcc {
    building: SliceDeque<CompoundOp>,
}

use BasicOp::*;
use CompoundOp::*;

const WINDOW_SIZE: usize = 127;

impl CompoundOpAcc {
    pub fn new() -> Self {
        Self {
            building: SliceDeque::with_capacity(WINDOW_SIZE + 1),
        }
    }

    pub fn feed(&mut self, basic_op: BasicOp) -> Option<CompoundOp> {
        self.building.push_back(CompoundOp::BasicOp(basic_op));

        match &self.building[..] {
            // Panic loop pattern
            [.., Set(value), BasicOp(LoopStart), BasicOp(LoopEnd)] if *value != 0 => {
                let value = *value;
                self.building.truncate_back(self.building.len() - 3);
                self.building.push_back(Panic(value));
            }
            // Zero cell pattern
            [.., BasicOp(LoopStart), BasicOp(ChangeBy(1 | u8::MAX)), BasicOp(LoopEnd)] => {
                self.building.truncate_back(self.building.len() - 3);

                if let Some(Zero) = self.building.back() {
                    // Don't add as it's redundant
                } else {
                    self.building.push_back(Zero);
                }
            }
            // Zero advance cell pattern
            [.., Zero, BasicOp(Shift(1))] => {
                self.building.truncate_back(self.building.len() - 2);

                // Merge with existing if there is one
                if let Some(ZeroAdvance(existing_amount)) = self.building.back() {
                    *self.building.back_mut().unwrap() = ZeroAdvance(existing_amount + 1);
                } else {
                    self.building.push_back(ZeroAdvance(1));
                }
            }
            // Zero retreat cell pattern
            [.., Zero, BasicOp(Shift(-1))] => {
                self.building.truncate_back(self.building.len() - 2);

                // Merge with existing if there is one
                if let Some(ZeroRetreat(existing_amount)) = self.building.back() {
                    *self.building.back_mut().unwrap() = ZeroRetreat(existing_amount + 1);
                } else {
                    self.building.push_back(ZeroRetreat(1));
                }
            }
            // Set cell pattern
            [.., Zero, BasicOp(ChangeBy(value))] => {
                let value = *value;
                self.building.truncate(self.building.len() - 2);

                // Merge with existing set instructions
                while let Some(Set(_)) = self.building.back() {
                    self.building.truncate(self.building.len() - 1);
                }

                self.building.push_back(Set(value));
            }
            // Equals algorithm
            [
                ..,
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(-1)),
                BasicOp(LoopEnd),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(1)),
                BasicOp(LoopStart),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(1)),
                Zero,
                BasicOp(LoopEnd),
            ] => {
                self.building.truncate_back(self.building.len() - 14);
                self.building.push_back(Equals);
            }
            // Not equals algorithm
            [
                ..,
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(-1)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                BasicOp(LoopStart),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(1)),
                Zero,
                BasicOp(LoopEnd),
            ] => {
                self.building.truncate_back(self.building.len() - 13);
                self.building.push_back(NotEquals);
            }
            // u8 shift left logical algorithm
            [
                ..,
                ZeroRetreat(1),
                BasicOp(LoopStart),
                BasicOp(Shift(-1)),
                MoveAdd(2),
                BasicOp(Shift(2)),
                BasicOp(LoopStart),
                BasicOp(Shift(-2)),
                BasicOp(ChangeBy(2)),
                BasicOp(Shift(2)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
            ] => {
                self.building.truncate_back(self.building.len() - 14);
                self.building.push_back(ShiftLeftLogical);
            }
            // u8 shift right logical algorithm
            [
                ..,
                ZeroAdvance(3),
                Zero,
                BasicOp(Shift(-4)),
                BasicOp(LoopStart),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(2)),
                BasicOp(Shift(-2)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(2)),
                BasicOp(ChangeBy(u8::MAX)),
                MoveAdd2(2, 3),
                BasicOp(Shift(3)),
                MoveAdd(-3),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopStart),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(2)),
                BasicOp(Shift(2)),
                BasicOp(ChangeBy(1)),
                BasicOp(LoopEnd),
                BasicOp(Shift(-4)),
                BasicOp(LoopEnd),
                BasicOp(Shift(3)),
                MoveAdd(-3),
                BasicOp(Shift(-1)),
                ZeroRetreat(1),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
            ] => {
                self.building.truncate_back(self.building.len() - 32);
                self.building.push_back(ShiftRightLogical);
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
            ]
            | [
                ..,
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(toward_amount)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(back_amount)),
                BasicOp(LoopEnd),
            ] if *toward_amount == -*back_amount => {
                let offset = *toward_amount;
                self.building.truncate_back(self.building.len() - 6);
                self.building.push_back(MoveAdd(offset));

                match &self.building[..] {
                    // Move set algorithm
                    [
                        ..,
                        BasicOp(Shift(toward_amount_plus_extra)),
                        CompoundOp::Zero,
                        BasicOp(Shift(back_amount)),
                        CompoundOp::MoveAdd(move_add_toward_amount),
                    ] if toward_amount_plus_extra.abs() >= back_amount.abs()
                        && -*back_amount == *move_add_toward_amount =>
                    {
                        let extra = toward_amount_plus_extra - -*back_amount;
                        let offset = -back_amount;

                        self.building.truncate_back(self.building.len() - 4);

                        if extra != 0 {
                            self.building.push_back(BasicOp(Shift(extra)));
                        }

                        self.building.push_back(MoveSet(offset));
                    }
                    // Dupe cell algorithm
                    [
                        ..,
                        ZeroAdvance(advance_amount),
                        Zero,
                        BasicOp(Shift(toward_shift)),
                        MoveAdd2(offset, offset_plus_1),
                        BasicOp(Shift(back_shift)),
                        MoveAdd(return_shift),
                    ] if -*toward_shift == *offset_plus_1
                        && *offset + 1 == *offset_plus_1
                        && -*toward_shift == *back_shift
                        && *toward_shift == *return_shift
                        && *advance_amount > 0 =>
                    {
                        let offset = -*offset;
                        let advance_amount = *advance_amount;

                        self.building.truncate_back(self.building.len() - 6);

                        if advance_amount > 1 {
                            self.building.push_back(ZeroAdvance(advance_amount - 1));
                        }

                        self.building.push_back(Dupe(offset));
                    }
                    // Less than algorithm
                    [
                        ..,
                        ZeroAdvance(zero_advance_amount),
                        Zero,
                        BasicOp(Shift(-2)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(1)),
                        Zero,
                        BasicOp(Shift(-2)),
                        MoveAdd2(2, 3),
                        BasicOp(Shift(2)),
                        MoveAdd(-2),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(1)),
                        BasicOp(LoopStart),
                        ZeroRetreat(1),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(3)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-1)),
                        Zero,
                        BasicOp(Shift(2)),
                        MoveAdd(-2),
                    ] if *zero_advance_amount > 0 => {
                        let zero_advance_amount = *zero_advance_amount;

                        self.building.truncate_back(self.building.len() - 26);

                        if zero_advance_amount > 1 {
                            self.building.push_back(ZeroAdvance(zero_advance_amount - 1));
                        }

                        self.building.push_back(LessThan);
                    }
                    // Greater than algorithm
                    [
                        ..,
                        ZeroAdvance(zero_advance_amount),
                        Zero,
                        BasicOp(Shift(-3)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(2)),
                        ZeroRetreat(1),
                        MoveAdd2(1, 2),
                        BasicOp(Shift(1)),
                        MoveAdd(-1),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(1)),
                        BasicOp(LoopStart),
                        ZeroRetreat(1),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(-1)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(2)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-3)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(2)),
                        MoveAdd(-2),
                    ] if *zero_advance_amount > 0 => {
                        let zero_advance_amount = *zero_advance_amount;

                        self.building.truncate_back(self.building.len() - 23);

                        if zero_advance_amount > 1 {
                            self.building.push_back(ZeroAdvance(zero_advance_amount - 1));
                        }

                        self.building.push_back(GreaterThan);
                    }
                    // Less than or equal algorithm
                    [
                        ..,
                        Set(1),
                        BasicOp(Shift(1)),
                        Zero,
                        BasicOp(Shift(-3)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(2)),
                        ZeroRetreat(1),
                        MoveAdd2(1, 2),
                        BasicOp(Shift(1)),
                        MoveAdd(-1),
                        BasicOp(Shift(1)),
                        BasicOp(LoopStart),
                        ZeroRetreat(1),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(-1)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(2)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-3)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(2)),
                        MoveAdd(-2),
                    ] => {
                        self.building.truncate_back(self.building.len() - 23);
                        self.building.push_back(LessThanEqual);
                    }
                    // Greater than or equal algorithm
                    [
                        ..,
                        Set(1),
                        BasicOp(Shift(1)),
                        Zero,
                        BasicOp(Shift(-2)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(1)),
                        Zero,
                        BasicOp(Shift(-2)),
                        MoveAdd2(2, 3),
                        BasicOp(Shift(2)),
                        MoveAdd(-2),
                        BasicOp(Shift(1)),
                        BasicOp(LoopStart),
                        ZeroRetreat(1),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(3)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-1)),
                        Zero,
                        BasicOp(Shift(2)),
                        MoveAdd(-2),
                    ] => {
                        self.building.truncate_back(self.building.len() - 26);
                        self.building.push_back(GreaterThanEqual);
                    }

                    // Bit and algorithm
                    [
                        ..,
                        Zero,
                        BasicOp(Shift(2)),
                        ZeroRetreat(1),
                        Set(248),
                        BasicOp(LoopStart),
                        BasicOp(ChangeBy(8)),
                        BasicOp(Shift(-2)),
                        ZeroRetreat(4),
                        Set(2),
                        BasicOp(Shift(-2)),
                        BasicOp(LoopStart),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        MoveAdd2(1, 3),
                        BasicOp(Shift(1)),
                        MoveAdd(-1),
                        BasicOp(Shift(4)),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(-1)),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(2)),
                        BasicOp(Shift(5)),
                        BasicOp(ChangeBy(254)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(1)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-5)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(4)),
                        MoveAdd(-4),
                        BasicOp(Shift(-2)),
                        Set(2),
                        BasicOp(Shift(-1)),
                        BasicOp(LoopStart),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(1)),
                        BasicOp(ChangeBy(u8::MAX)),
                        MoveAdd2(1, 3),
                        BasicOp(Shift(1)),
                        MoveAdd(-1),
                        BasicOp(Shift(3)),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(-1)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(1)),
                        BasicOp(ChangeBy(254)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(1)),
                        BasicOp(Shift(-2)),
                        BasicOp(ChangeBy(2)),
                        BasicOp(Shift(3)),
                        BasicOp(ChangeBy(1)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-4)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(3)),
                        MoveAdd(-3),
                        BasicOp(Shift(2)),
                        BasicOp(LoopStart),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(Shift(1)),
                        MoveAdd(-2),
                        BasicOp(Shift(-1)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(1)),
                        ZeroAdvance(1),
                        MoveAdd2(-1, -2),
                        BasicOp(Shift(-1)),
                        MoveAdd(1),
                        BasicOp(Shift(-1)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(-1)),
                        MoveAdd(-1),
                        BasicOp(Shift(-1)),
                        BasicOp(LoopStart),
                        BasicOp(Shift(1)),
                        BasicOp(ChangeBy(2)),
                        BasicOp(Shift(-1)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(2)),
                        BasicOp(ChangeBy(u8::MAX)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(-1)),
                        MoveAdd(4),
                        BasicOp(Shift(3)),
                        BasicOp(ChangeBy(249)),
                        BasicOp(LoopEnd),
                        BasicOp(Shift(1)),
                        MoveAdd(-9),
                    ] => {
                        self.building.truncate_back(self.building.len() - 96);
                        self.building.push_back(BitAnd);
                    }
                    _ => {}
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
            // Bit negate algorithm
            [
                ..,
                MoveAdd(1),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(LoopStart),
                BasicOp(Shift(-1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
            ] => {
                self.building.truncate_back(self.building.len() - 9);
                self.building.push_back(BitNeg);
            }
            // Divmod algorithm
            [
                ..,
                ZeroAdvance(3),
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
                MoveAdd(-1),
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
                MoveAdd(-1),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(2)),
                BasicOp(LoopEnd),
                BasicOp(Shift(neg_5_plus_extra)),
            ] => {
                let default_shift_amount = -2;
                let shift_amount = neg_5_plus_extra + 5 + default_shift_amount;
                self.building.truncate_back(self.building.len() - 36);
                self.building.push_back(WellBehavedDivMod(shift_amount));
            }
            // Print Static pattern
            [.., Set(initial_letter), BasicOp(Output(letter_count))] => {
                let initial_letter = *initial_letter;
                let letter_count = *letter_count as usize;

                self.building.truncate_back(self.building.len() - 2);

                // Merge with previous if exists
                if let Some(PrintStatic(content)) = self.building.back_mut() {
                    for _ in 0..letter_count {
                        content.push(initial_letter);
                    }
                } else {
                    self.building
                        .push_back(PrintStatic(vec![initial_letter; letter_count as usize]));
                }
            }
            // Print Static pattern (continuation)
            [
                ..,
                PrintStatic(string),
                BasicOp(ChangeBy(letter_change_amount)),
                BasicOp(Output(letter_count)),
            ] => {
                let new_letter = string.last().unwrap().wrapping_add(*letter_change_amount);
                let letter_count = *letter_count;

                self.building.truncate_back(self.building.len() - 2);

                // Re pattern match with mut
                if let PrintStatic(string) = self.building.back_mut().unwrap() {
                    for _ in 0..letter_count {
                        string.push(new_letter);
                    }
                } else {
                    panic!();
                }
            }
            // Move cell dynamic u8 algorithm
            [
                ..,
                ZeroAdvance(2),
                Zero,
                BasicOp(Shift(-3)),
                BasicOp(LoopStart),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(1)),
                BasicOp(ChangeBy(1)),
                BasicOp(Shift(-3)),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(LoopEnd),
                BasicOp(Shift(3)),
                MoveAdd(-3),
                BasicOp(Shift(-3)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(3)),
                ZeroRetreat(1),
                MoveAdd(1),
                BasicOp(Shift(-1)),
                MoveAdd(1),
                BasicOp(Shift(-1)),
                MoveAdd(1),
                BasicOp(Shift(-1)),
                MoveAdd(1),
                BasicOp(Shift(2)),
                BasicOp(LoopEnd),
                BasicOp(Shift(-1)),
                MoveSet(neg_offset),
                BasicOp(Shift(3)),
                MoveAdd(-2),
                BasicOp(Shift(-2)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                MoveAdd(-1),
                BasicOp(Shift(1)),
                MoveAdd(-1),
                BasicOp(Shift(-2)),
                BasicOp(LoopEnd),
                BasicOp(Shift(1)),
                MoveAdd(-2),
                BasicOp(Shift(neg_2_plus_extra)),
            ] if *neg_2_plus_extra <= -2 && *neg_offset < 0 => {
                // We ignore normal first shift right instruction,
                // so offset will be 1 less than normal
                let offset = -*neg_offset as u64 - 1;
                let neg_2_plus_extra = *neg_2_plus_extra;

                self.building.truncate(self.building.len() - 44);

                if neg_2_plus_extra != -2 {
                    self.building.push_back(BasicOp(Shift(neg_2_plus_extra - -2)));
                }

                self.building.push_back(MoveCellDynamicU8(offset));
            }
            // Copy cell dynamic u8 algorithm
            [
                ..,
                MoveAdd(-1),
                Dupe(-1),
                BasicOp(Shift(-2)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(2)),
                ZeroRetreat(1),
                MoveAdd(1),
                BasicOp(Shift(-1)),
                MoveAdd(1),
                BasicOp(Shift(1)),
                BasicOp(LoopEnd),
                Zero,
                BasicOp(Shift(2)),
                Zero,
                BasicOp(Shift(neg_2_plus_neg_offset)),
                MoveAdd2(move_offset, pos_2_plus_pos_offset),
                BasicOp(Shift(shift_pos_2_plus_pos_offset)),
                MoveAdd(move_neg_2_plus_neg_offset),
                BasicOp(Shift(-1)),
                BasicOp(LoopStart),
                BasicOp(ChangeBy(u8::MAX)),
                BasicOp(Shift(-1)),
                MoveAdd(-1),
                MoveAdd(-1),
                BasicOp(Shift(-1)),
                BasicOp(LoopEnd),
            ] if *neg_2_plus_neg_offset == *move_neg_2_plus_neg_offset
                && *pos_2_plus_pos_offset == *shift_pos_2_plus_pos_offset
                && -*neg_2_plus_neg_offset == *pos_2_plus_pos_offset
                && *pos_2_plus_pos_offset - 2 == *move_offset =>
            {
                // We ignore normal first shift left instruction,
                // so offset will be 1 less than normal
                let offset = *move_offset as u64;
                self.building.truncate_back(self.building.len() - 27);
                self.building.push_back(CopyCellDynamicU8(offset));
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
