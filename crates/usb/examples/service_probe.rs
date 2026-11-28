//! Diagnose a camera's service protocol: handshake, identity, and which
//! addresses the debug memory command admits. Read-only.
use fujimod_usb::{
    connection::{Connection, DeviceFilter, TransferOptions, discover, interfaces},
    ff80::{
        Session,
        commands::{
            configuration::{
                ConfigurationGeometry, ConfigurationLocation, ConfigurationView, ReadConfiguration,
            },
            info::InfoSelector,
            memory::{Address, DebugMemoryLimits, MemoryRange, ReadMemory},
        },
    },
};
use std::{num::NonZeroU32, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_lite::future::block_on(run())
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let devices = discover(DeviceFilter::Diagnostic).await?;
    let device = devices.into_iter().next().ok_or("no service camera")?;
    println!(
        "device {:04x}:{:04x} bus={} addr={}",
        device.vendor_id(),
        device.product_id(),
        device.bus_id(),
        device.device_address()
    );
    let mut session = None;
    for selection in interfaces(&device).await? {
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
                    session = Some(candidate);
                    break;
                }
                Err(error) => println!("handshake failed: {error}"),
            },
            Err(error) => println!("open failed: {error}"),
        }
    }
    let mut session = session.ok_or("no service session")?;
    let result = diagnose(&mut session).await;
    println!("session closed: {:?}", session.close().await);
    result
}
async fn diagnose(session: &mut Session) -> Result<(), Box<dyn std::error::Error>> {
    println!("ping {:02x?}", session.ping().await?);
    for selector in [
        InfoSelector::Vendor,
        InfoSelector::Manufacturer,
        InfoSelector::Product,
        InfoSelector::InternalVersion,
        InfoSelector::DeviceType,
        InfoSelector::Serial,
        InfoSelector::Framework,
    ] {
        let name = format!("{selector:?}");
        match session.device_info(selector).await {
            Ok(info) => println!("{name}: {:?}", info.text),
            Err(error) => println!("{name}: error {error}"),
        }
    }
    let limits = DebugMemoryLimits {
        read: NonZeroU32::new(256).expect("read"),
        write: NonZeroU32::new(256).expect("write"),
    };
    for (address, length) in [
        (0x0000_0000u32, 4u32),
        (0x0000_0000, 256),
        (0x0010_0000, 4),
        (0x0101_376c, 4),
        (0x0266_8000, 4),
        (0x0e29a98, 4),
        (0x1000_0000, 4),
        (0x2000_0000, 4),
        (0xea7eb8, 4),
    ] {
        let range = MemoryRange::new(
            Address(address),
            NonZeroU32::new(length).expect("length"),
        )?;
        let command = ReadMemory::new(range, limits)?;
        match session.read_memory(command).await {
            Ok(bytes) => println!("read {address:#010x}+{length:#x} ok {bytes:02x?}"),
            Err(error) => println!("read {address:#010x}+{length:#x} {error}"),
        }
    }
    for view in [ConfigurationView::Storage, ConfigurationView::Ram] {
        match session.configuration_geometry(view).await {
            Ok(ConfigurationGeometry {
                capacity,
                page_bytes,
                ..
            }) => {
                println!("{view:?} capacity {capacity:#x} page {page_bytes}");
                for offset in [0u32, 0x200, 0x201, 14_324, 0x202, 0x203] {
                    let command = ReadConfiguration::new(
                        ConfigurationLocation {
                            view,
                            page: 0,
                            offset,
                        },
                        NonZeroU32::new(8).expect("length"),
                        page_bytes,
                    )?;
                    match session.read_configuration(command).await {
                        Ok(bytes) => println!("  {view:?} {offset:#x} {bytes:02x?}"),
                        Err(error) => println!("  {view:?} {offset:#x} {error}"),
                    }
                }
            }
            Err(error) => println!("{view:?} geometry failed: {error}"),
        }
    }
    Ok(())
}
