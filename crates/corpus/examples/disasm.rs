//! Disassemble a raw slice of one architecture, for RE notes.
//!
//! ```sh
//! cargo run -p fujimod-corpus --example disasm -- FILE BASE OFFSET LENGTH [arm|thumb]
//! ```
use capstone::prelude::*;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (path, base, offset, length, thumb) = (
        args.first().ok_or("usage: FILE BASE OFFSET LENGTH [arm|thumb]")?,
        args.get(1).ok_or("BASE")?,
        args.get(2).ok_or("OFFSET")?,
        args.get(3).ok_or("LENGTH")?,
        args.get(4).map(String::as_str) == Some("thumb"),
    );
    let number = |value: &str| u64::from_str_radix(value.trim_start_matches("0x"), 16);
    let base = number(base)?;
    let offset = number(offset)?;
    let length = number(length)? as usize;
    let bytes = fs::read(path)?;
    let start = offset as usize;
    let slice = bytes
        .get(start..start + length)
        .ok_or("slice escapes the file")?;
    let cs = if thumb {
        Capstone::new().arm().mode(arch::arm::ArchMode::Thumb).build()?
    } else {
        Capstone::new().arm().mode(arch::arm::ArchMode::Arm).build()?
    };
    for instruction in cs.disasm_all(slice, base + offset)?.iter() {
        println!(
            "{:08x}  {:8} {}",
            instruction.address(),
            instruction.mnemonic().unwrap_or(""),
            instruction.op_str().unwrap_or("")
        );
    }
    Ok(())
}
