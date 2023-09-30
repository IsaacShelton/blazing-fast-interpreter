use anyhow::{anyhow, Result};

#[derive(Copy, Clone, Debug)]
pub enum BasicOp {
    ChangeBy(u8),
    Shift(i64),
    LoopStart,
    LoopEnd,
    Input(u64),
    Output(u64),
}

use BasicOp::*;

pub struct BasicOpAcc {
    building: Option<BasicOp>,
}

impl BasicOpAcc {
    pub fn new() -> Self {
        Self { building: None }
    }

    pub fn feed(&mut self, op: BasicOp) -> Option<BasicOp> {
        match (self.building, op) {
            (None, _) => {
                self.building = Some(op);
            }
            (Some(ChangeBy(current)), ChangeBy(new)) => {
                self.building = Some(ChangeBy(current.wrapping_add(new)));
            }
            (Some(Shift(current)), Shift(new)) => {
                self.building = Some(Shift(current + new));
            }
            (Some(Input(current)), Input(new)) => {
                self.building = Some(Input(current + new));
            }
            (Some(Output(current)), Output(new)) => {
                self.building = Some(Output(current + new));
            }
            (Some(_), _) => {
                return std::mem::replace(&mut self.building, Some(op));
            }
        }

        None
    }

    pub fn feed_byte(&mut self, byte: u8) -> Result<Option<BasicOp>> {
        match byte {
            b' ' | b'\n' => return Ok(None),
            _ => (),
        };

        let op = match byte {
            b'+' => BasicOp::ChangeBy(1),
            b'-' => BasicOp::ChangeBy(u8::MAX),
            b'[' => BasicOp::LoopStart,
            b']' => BasicOp::LoopEnd,
            b'<' => BasicOp::Shift(-1),
            b'>' => BasicOp::Shift(1),
            b',' => BasicOp::Input(1),
            b'.' => BasicOp::Output(1),
            _ => return Err(anyhow!("Invalid character")),
        };

        Ok(self.feed(op))
    }

    pub fn finalize(&mut self) -> Option<BasicOp> {
        std::mem::replace(&mut self.building, None)
    }
}
