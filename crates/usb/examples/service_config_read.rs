//! Read a bounded range of one configuration view, in page-sized chunks.
//!
//! Read-only.
//!
//! ```sh
//! cargo run -p fujimod-usb --example service_config_read -- VIEW OFFSET LENGTH [OUT]
//! ```
//!
//! VIEW is 0 (storage) or 1 (ram); offsets and lengths are hexadecimal.
use fujimod_usb::{
    connection::{Connection, DeviceFilter, TransferOptions, discover, interfaces},
    ff80::{
        Session,
        commands::configuration::{ConfigurationLocation, ConfigurationView, ReadConfiguration},
    },
};
use std::{fs, num::NonZeroU32, path::Path, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_lite::future::block_on(run())
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let view = match args.first().map(String::as_str) {
        Some("1") | Some("ram") => ConfigurationView::Ram,
        _ => ConfigurationView::Storage,
    };
    let offset = u32::from_str_radix(args.get(1).ok_or("usage: VIEW OFFSET LENGTH [OUT]")?.trim_start_matches("0x"), 16)?;
    let length = u32::from_str_radix(args.get(2).ok_or("usage: VIEW OFFSET LENGTH [OUT]")?.trim_start_matches("0x"), 16)?;
    let out = args.get(3).map(String::as_str);
    let device = discover(DeviceFilter::Diagnostic)
        .await?
        .into_iter()
        .next()
        .ok_or("no service camera")?;
    let mut session = None;
    for selection in interfaces(&device).await? {
        if let Ok(connection) = Connection::open(
            &device,
            selection,
            TransferOptions {
                timeout: Duration::from_secs(10),
                max_reply_bytes: NonZeroU32::new(8 << 20).expect("budget"),
            },
        )
        .await
        {
            if let Ok(candidate) = connection.begin_session(None).await {
                session = Some(candidate);
                break;
            }
        }
    }
    let mut session = session.ok_or("no service session")?;
    let result = read(&mut session, view, offset, length, out).await;
    println!("session closed: {:?}", session.close().await);
    result
}
async fn read(
    session: &mut Session,
    view: ConfigurationView,
    offset: u32,
    length: u32,
    out: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let geometry = session.configuration_geometry(view).await?;
    let page = geometry.page_bytes;
    println!("{view:?} capacity {:#x} page {page}", geometry.capacity);
    let mut bytes = Vec::with_capacity(length as usize);
    let started = std::time::Instant::now();
    while (bytes.len() as u32) < length {
        let at = offset + bytes.len() as u32;
        let want = (length - bytes.len() as u32).min(page.get());
        let command = ReadConfiguration::new(
            ConfigurationLocation { view, page: 0, offset: at },
            NonZeroU32::new(want).ok_or("zero length")?,
            page,
        )?;
        let chunk = session.read_configuration(command).await?;
        if chunk.len() != want as usize {
            return Err(format!("short read at {at:#x}: {} of {want}", chunk.len()).into());
        }
        bytes.extend_from_slice(&chunk);
    }
    for (index, chunk) in bytes.chunks(16).enumerate() {
        let line = chunk.iter().map(|v| format!("{v:02x}")).collect::<Vec<_>>().join(" ");
        println!("{:08x}  {line}", offset + (index * 16) as u32);
    }
    println!("{} bytes in {:.1}s", bytes.len(), started.elapsed().as_secs_f32());
    if let Some(path) = out {
        fs::write(Path::new(path), &bytes)?;
        println!("wrote {path}");
    }
    Ok(())
}
