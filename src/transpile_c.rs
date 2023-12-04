use crate::{basic_op::BasicOp, compound_op::CompoundOp, interpreter::CELL_COUNT, interpreter_op::InterpreterOp};
use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub fn transpile_c<'a>(ops: impl Iterator<Item = &'a InterpreterOp>, output_filename: &str) -> Result<()> {
    let file = File::create(output_filename)?;
    let mut f = BufWriter::new(file);

    f.write(b"#include <stdio.h>\n")?;
    f.write(b"#include <stdlib.h>\n")?;
    f.write(b"#include <string.h>\n")?;
    f.write(b"static inline void put(unsigned char c){ putchar((char) c); }\n")?;
    f.write(
        b"static inline unsigned char get(void){ char c = getc(stdin); return c != EOF ? (unsigned char) c : 0; }\n",
    )?;

    f.write(b"int main(){\n")?;
    f.write(format!("unsigned char *m = malloc({});\n", CELL_COUNT).as_bytes())?;
    f.write(b"size_t i = 0;\n")?;
    f.write(format!("memset(m, 0, {});\n", CELL_COUNT).as_bytes())?;

    for op in ops {
        match op {
            InterpreterOp::LoopStart(_) => {
                f.write(b"while(m[i]){\n")?;
            }
            InterpreterOp::LoopEnd(_) => {
                f.write(b"}\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::ChangeBy(amount))) => {
                f.write(format!("m[i] += {};\n", amount).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Shift(amount))) => {
                if *amount >= 0 {
                    f.write(format!("i += {};\n", amount).as_bytes())?;
                } else {
                    f.write(format!("i -= {};\n", -amount).as_bytes())?;
                }
            }
            InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Input(count))) => {
                for _ in 0..*count {
                    f.write(b"m[i] = get();\n")?;
                }
            }
            InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::Output(count))) => {
                for _ in 0..*count {
                    f.write(b"put(m[i]);\n")?;
                }
                f.write(b"fflush(stdout);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::BasicOp(BasicOp::LoopStart | BasicOp::LoopEnd)) => {
                return Err(anyhow!("[error] Cannot transpile unprocessed loop instruction"));
            }
            InterpreterOp::CompoundOp(CompoundOp::Panic(value)) => {
                f.write(format!("m[i] = {};\n", value).as_bytes())?;
                f.write(b"exit(m[i]);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::Zero) => {
                f.write(b"m[i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::ZeroAdvance(amount)) => {
                f.write(format!("memset(&m[i], 0, {});\n", *amount).as_bytes())?;
                f.write(format!("i += {};\n", *amount).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::ZeroRetreat(amount)) => {
                f.write(format!("memset(&m[i - {}], 0, {});\n", *amount - 1, *amount).as_bytes())?;
                f.write(format!("i -= {};\n", *amount).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::Set(value)) => {
                f.write(format!("m[i] = {};\n", value).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::Dupe(offset)) => {
                f.write(format!("m[i] = m[i + {}];\n", *offset).as_bytes())?;
                f.write("m[++i] = 0;\n".as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::BitAnd) => {
                // Warning: Unsound

                // a b ? ? ? ? ? ?
                //               ^

                f.write(b"m[i - 7] &= m[i - 6];\n")?;
                f.write(b"m[i - 6] = 0;\n")?;
                f.write(b"i += 2;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::WellBehavedDivMod(shift_amount)) => {
                f.write(b"if(m[i - 1] == 0){\n")?;
                f.write(b"m[i] = 0;\n")?;
                f.write(b"m[i + 1] = 0;\n")?;
                f.write(b"} else {\n")?;
                f.write(b"m[i] = m[i - 2] % m[i - 1];\n")?;
                f.write(b"m[i + 1] = m[i - 2] / m[i - 1];\n")?;
                f.write(b"}\n")?;
                f.write(b"m[i - 1] = m[i - 1] - m[i + 1];\n")?;
                f.write(b"m[i - 2] = 0;\n")?;
                f.write(b"m[i + 2] = 0;\n")?;
                f.write(b"m[i + 3] = 0;\n")?;
                f.write(format!("i += {};\n", shift_amount).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::BitNeg) => {
                f.write(b"m[i] = ~m[i];\n")?;
                f.write(b"m[++i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::Equals) => {
                f.write(b"m[i] = (m[i] == m[i + 1]);\n")?;
                f.write(b"m[++i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::NotEquals) => {
                f.write(b"m[i] = (m[i] != m[i + 1]);\n")?;
                f.write(b"m[++i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::ShiftLeftLogical) => {
                f.write(b"m[i - 2] = (m[i - 1] >= 8) ? 0 : m[i - 2] << m[i - 1];\n")?;
                f.write(b"m[i - 1] = 0;\n")?;
                f.write(b"m[i--] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::ShiftRightLogical) => {
                f.write(b"m[i - 2] = (m[i - 1] >= 8) ? 0 : m[i - 2] >> m[i - 1];\n")?;
                f.write(b"memset(&m[--i], 0, 5);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::LessThan) => {
                f.write(b"m[i - 2] = m[i - 2] < m[i - 1];\n")?;
                f.write(b"memset(&m[i - 1], 0, 3);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::GreaterThan) => {
                f.write(b"m[i - 2] = m[i - 2] > m[i - 1];\n")?;
                f.write(b"memset(&m[i - 1], 0, 3);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::LessThanEqual) => {
                f.write(b"m[i - 2] = m[i - 2] <= m[i - 1];\n")?;
                f.write(b"memset(&m[i - 1], 0, 3);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::GreaterThanEqual) => {
                f.write(b"m[i - 2] = m[i - 2] >= m[i - 1];\n")?;
                f.write(b"memset(&m[i - 1], 0, 3);\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveAdd(offset)) => {
                f.write(format!("m[i + {}] += m[i];\n", offset).as_bytes())?;
                f.write(b"m[i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveSet(offset)) => {
                f.write(format!("m[i + {}] = m[i];\n", offset).as_bytes())?;
                f.write(b"m[i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveAdd2(offset1, offset2)) => {
                f.write(format!("m[i + {}] += m[i];\n", offset1).as_bytes())?;
                f.write(format!("m[i + {}] += m[i];\n", offset2).as_bytes())?;
                f.write(b"m[i] = 0;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::PrintStatic(content)) => {
                for c in content {
                    f.write(format!("put({});\n", *c).as_bytes())?;
                }
                f.write("fflush(stdout);\n".as_bytes())?;
                f.write(format!("m[i] = {};\n", *content.last().unwrap()).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU8(offset)) => {
                // Warning: Unsound
                f.write(format!("m[i - {} + m[i - 1]] = m[i - 2];\n", 3 + offset).as_bytes())?;
                f.write(b"m[i - 2] = m[i - 1];\n")?;
                f.write(b"i -= 2;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU16(offset)) => {
                // Warning: Unsound
                f.write(
                    format!(
                        "m[  i - {} + (size_t) m[i - 2] | ((size_t) m[i - 1] << 8)  ] = m[i - 3];\n",
                        *offset
                    )
                    .as_bytes(),
                )?;
                f.write(b"m[i - 3] = m[i - 2];\n")?;
                f.write(b"m[i - 2] = m[i - 1];\n")?;
                f.write(b"i -= 3;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::CopyCellDynamicU8(offset)) => {
                // Warning: Unsound
                f.write(format!("m[i - 1] = m[  i - {} + m[i - 1]  ];\n", *offset + 1).as_bytes())?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveCellDynamicU32(offset)) => {
                // Warning: Unsound
                f.write(
                    format!(
                        "m[  i - {} + (size_t) m[i - 4] | ((size_t) m[i - 3] << 8) | ((size_t) m[i - 2] << 16) | ((size_t) m[i - 1] << 24)  ] = m[i - 5];\n",
                        *offset
                    )
                    .as_bytes(),
                )?;
                f.write(b"m[i - 5] = m[i - 4];\n")?;
                f.write(b"m[i - 4] = m[i - 3];\n")?;
                f.write(b"m[i - 3] = m[i - 2];\n")?;
                f.write(b"m[i - 2] = m[i - 1];\n")?;
                f.write(b"i -= 5;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::CopyCellDynamicU32(offset)) => {
                // Warning: Unsound

                f.write(
                    format!("m[i - 4] = m[  i - {} + (size_t) m[i - 4] | ((size_t) m[i - 3] << 8) | ((size_t) m[i - 2] << 16) | ((size_t) m[i - 1] << 24)  ];\n", *offset)
                    .as_bytes()
                    )?;

                f.write(b"i -= 3;\n")?;
            }
            InterpreterOp::CompoundOp(CompoundOp::MoveCellsStaticReverse(offset, count)) => {
                f.write(format!("memmove(&m[i + {}], &m[i - {}], {});\n", *offset - *count as i64 + 1, *count - 1, *count).as_bytes())?;
                f.write(format!("i -= {};\n", *count).as_bytes())?;
            }
        }
    }

    f.write(b"free(m);\n")?;
    f.write(b"return 0;\n")?;
    f.write(b"}\n")?;
    Ok(())
}
