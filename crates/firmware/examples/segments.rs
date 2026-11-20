//! List a firmware image's decompressed candidates and their load bases.
//!
//! \`\`\`sh
//! cargo run -p fujimod-firmware --example segments -- FWUP0008.DAT
//! \`\`\`
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: segments DAT")?;
    let raw = fs::read(path)?;
    let outer = fujimod_firmware::outer_header(&raw)?;
    let payload = fujimod_firmware::extent(
        &raw,
        outer.payload_offset as u64,
        (raw.len() - outer.payload_offset) as u64,
    )?;
    for (index, candidate) in fujimod_firmware::scan(payload).iter().enumerate() {
        let base = candidate
            .header
            .as_ref()
            .map(|header| format!("{:#x}", header.base));
        println!(
            "{index:3} offset {:#08x} decoded {:>9} compression {:?} base {}",
            candidate.offset,
            candidate.decoded.len(),
            candidate.compression,
            base.unwrap_or_else(|| "-".into()),
        );
    }
    Ok(())
}
