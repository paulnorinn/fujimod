//! Read physical memory from a camera in service mode.
//!
//! Read-only: this example never issues a write, erase or service transition.
//!
//! ```sh
//! cargo run -p fujimod-usb --example service_read -- OUT_DIR ADDRESS:LENGTH...
//! ```
//!
//! Addresses and lengths are hexadecimal, `0x` optional.
use fujimod_usb::{
    connection::{Connection, DeviceFilter, InterfaceSelection, TransferOptions, discover, interfaces},
    ff80::{
        Session,
        commands::memory::{Address, DebugMemoryLimits, MemoryRange},
    },
    operations::memory,
};
use std::{fs, num::NonZeroU32, path::Path, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_lite::future::block_on(run())
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (out, specs) = args.split_first().ok_or("usage: service_read OUT_DIR ADDRESS:LENGTH...")?;
    let out = Path::new(out);
    fs::create_dir_all(out)?;
    let devices = discover(DeviceFilter::Diagnostic).await?;
    println!("discovered {} camera(s)", devices.len());
    for device in &devices {
        println!(
            "  {:04x}:{:04x} bus={} address={} serial={:?}",
            device.vendor_id(),
            device.product_id(),
            device.bus_id(),
            device.device_address(),
            device.serial_number(),
        );
    }
    let device = devices.into_iter().next().ok_or("no Fujifilm camera in service mode")?;
    let mut session = None;
    for selection in interfaces(&device).await? {
        let InterfaceSelection {
            configuration,
            interface,
            alternate,
        } = selection;
        match Connection::open(
            &device,
            selection,
            TransferOptions {
                timeout: Duration::from_secs(10),
                max_reply_bytes: NonZeroU32::new(8 << 20).expect("budget"),
            },
        )
        .await
        {
            Ok(connection) => match connection.begin_session(None).await {
                Ok(candidate) => {
                    println!(
                        "session on configuration {configuration} interface {interface} alternate {alternate}: id {} packet {}",
                        candidate.info().id,
                        candidate.info().advertised_packet_bytes,
                    );
                    session = Some(candidate);
                    break;
                }
                Err(error) => println!("interface {interface} handshake failed: {error}"),
            },
            Err(error) => println!("interface {interface} open failed: {error}"),
        }
    }
    let mut session = session.ok_or("no service session")?;
    let result = read_all(&mut session, specs, out).await;
    println!("session closed: {:?}", session.close().await);
    result
}
async fn read_all(
    session: &mut Session,
    specs: &[String],
    out: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let limits = DebugMemoryLimits {
        read: NonZeroU32::new(1 << 20).expect("read limit"),
        write: NonZeroU32::new(1 << 20).expect("write limit"),
    };
    for spec in specs {
        let (address, length) = spec.split_once(':').ok_or("spec must be ADDRESS:LENGTH")?;
        let address = u32::from_str_radix(address.trim_start_matches("0x"), 16)?;
        let length = u32::from_str_radix(length.trim_start_matches("0x"), 16)?;
        let range = MemoryRange::new(
            Address(address),
            NonZeroU32::new(length).ok_or("zero length")?,
        )?;
        let bytes = memory::read_range(session, range, limits).await?;
        let path = out.join(format!("{address:08x}-{length:x}.bin"));
        fs::write(&path, &bytes)?;
        println!("read {address:#010x} +{length:#x} -> {} ({} bytes)", path.display(), bytes.len());
    }
    Ok(())
}
