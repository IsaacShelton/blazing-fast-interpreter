use std::fmt;

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
    trailing: Option<(BasicOp, usize)>,
    number: Option<usize>,
}

impl BasicOpAcc {
    pub fn new() -> Self {
        Self {
            building: None,
            trailing: None,
            number: None,
        }
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
                return std::mem::replace(&mut self.building, Some(op)).and_then(Self::filter);
            }
        }

        None
    }

    pub fn feed_byte(&mut self, byte: u8) -> Result<Option<BasicOp>> {
        match byte {
            b' ' | b'\n' => return Ok(None),
            _ => (),
        };

        let count = self.number.unwrap_or(1);

        let op = match byte {
            b'+' => BasicOp::ChangeBy(count as u8),
            b'-' => BasicOp::ChangeBy(0 - count as u8),
            b'[' => {
                let op = BasicOp::LoopStart;
                if count > 1 {
                    self.trailing = Some((op, count - 1));
                }
                op
            }
            b']' => {
                let op = BasicOp::LoopEnd;
                if count > 1 {
                    self.trailing = Some((op, count - 1));
                }
                op
            }
            b'<' => BasicOp::Shift(-(count as i64)),
            b'>' => BasicOp::Shift(count as i64),
            b',' => BasicOp::Input(count as u64),
            b'.' => BasicOp::Output(count as u64),
            b'0'..=b'9' => {
                let digit = (byte - b'0') as usize;

                self.number = Some(match self.number {
                    Some(count) => 10 * count + digit,
                    None => digit,
                });

                return Ok(None);
            }
            _ => return Err(anyhow!("Invalid character")),
        };

        self.number = None;
        Ok(self.feed(op))
    }

    pub fn continued(&mut self) -> Option<BasicOp> {
        match self.trailing {
            Some((_, 0)) => {
                self.trailing = None;
                None
            }
            Some((op, 1)) => {
                self.trailing = None;
                Some(op)
            }
            Some((op, count)) => {
                self.trailing = Some((op, count - 1));
                Some(op)
            }
            None => None,
        }
    }

    pub fn filter(op: BasicOp) -> Option<BasicOp> {
        match op {
            BasicOp::Shift(0) => None,
            BasicOp::ChangeBy(0) => None,
            _ => Some(op),
        }
    }

    pub fn finalize(&mut self) -> Option<BasicOp> {
        let built = std::mem::replace(&mut self.building, None).and_then(Self::filter);

        match built {
            Some(_) => built,
            None => self.continued(),
        }
    }
}

impl fmt::Display for BasicOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ChangeBy(amount) => {
                if amount > 128 {
                    let count = u8::MAX.wrapping_sub(amount - 1);

                    if count == 1 {
                        write!(f, "-")
                    } else {
                        write!(f, "{}-", count)
                    }
                } else if amount > 0 {
                    if amount == 1 {
                        write!(f, "+")
                    } else {
                        write!(f, "{}+", amount)
                    }
                } else {
                    fmt::Result::Ok(())
                }
            }
            Shift(amount) => {
                if amount > 0 {
                    if amount == 1 {
                        write!(f, ">")
                    } else {
                        write!(f, "{}>", amount)
                    }
                } else if amount < 0 {
                    if amount == -1 {
                        write!(f, "<")
                    } else {
                        write!(f, "{}<", -amount)
                    }
                } else {
                    fmt::Result::Ok(())
                }
            }
            LoopStart => write!(f, "["),
            LoopEnd => write!(f, "]"),
            Input(count) => {
                if count == 1 {
                    write!(f, ",")
                } else {
                    write!(f, "{},", count)
                }
            }
            Output(count) => {
                if count == 1 {
                    write!(f, ".")
                } else {
                    write!(f, "{}.", count)
                }
            }
        }
    }
}
