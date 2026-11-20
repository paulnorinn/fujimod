//! Write the X-Processor Pro execution and device graph of a bundled run
//! configuration as JSON, with explicit assets.
//!
//! Used to rebuild the prepared browser document after a catalog change.
//! \`\`\`sh
//! cargo run -p fujimod-emulator-cli -- xpp-definition CONFIG OUT.json DAT \
//!     ASSET_NAME=PATH...
//! \`\`\`
use std::{collections::BTreeMap, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (configuration, out, assets) = (
        args.first().ok_or("usage: CONFIG OUT.json ASSET_NAME=PATH...")?,
        args.get(1).ok_or("OUT.json")?,
        &args[2..],
    );
    let registers: serde_json::Value = serde_json::from_str(&format!(
        "[[[\"Pc\",0]],[[\"Pc\",0]]]"
    ))?;
    let mut loaded: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for spec in assets {
        let (name, path) = spec.split_once('=').ok_or("asset must be NAME=PATH")?;
        loaded.insert(name.to_owned(), fs::read(path)?);
    }
    let definition = fujimod_emulator_boot::xpp::definition(
        configuration,
        &[],
        registers,
        &[],
        &loaded,
        None,
    )?;
    fs::write(PathBuf::from(out), serde_json::to_vec_pretty(&definition)?)?;
    println!("wrote {out}");
    Ok(())
}
