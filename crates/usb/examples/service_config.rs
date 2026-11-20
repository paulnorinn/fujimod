//! Dump a camera's configuration views over the service protocol.
//!
//! Read-only: no write, erase or service transition is ever issued.
//!
//! ```sh
//! cargo run -p fujimod-usb --example service_config -- OUT_DIR
//! ```
use fujimod_usb::{
    connection::{Connection, DeviceFilter, InterfaceSelection, TransferOptions, discover, interfaces},
    ff80::{
        Session,
        commands::configuration::{ConfigurationLocation, ConfigurationView, ReadConfiguration},
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    num::NonZeroU32,
    path::{Path, PathBuf},
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_lite::future::block_on(run())
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args.first().ok_or("usage: service_config OUT_DIR [CHUNK_BYTES]")?.clone();
    let chunk_bytes = match args.get(1) {
        Some(value) => u32::from_str_radix(value.trim_start_matches("0x"), 16)?,
        None => 0x4000,
    };
    let storage = args.iter().any(|value| value == "--storage");
    let out = PathBuf::from(out);
    fs::create_dir_all(&out)?;
    let devices = discover(DeviceFilter::Diagnostic).await?;
    let device = devices.into_iter().next().ok_or("no service camera")?;
    let mut session = None;
    for selection in interfaces(&device).await? {
        let InterfaceSelection { .. } = selection;
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
    let result = dump(&mut session, &out, chunk_bytes, storage).await;
    println!("session closed: {:?}", session.close().await);
    result
}
async fn dump(session: &mut Session, out: &Path, chunk_bytes: u32, storage: bool) -> Result<(), Box<dyn std::error::Error>> {
    // The RAM view is the running copy; the storage view reads flash and is slow.
    let mut views = vec![ConfigurationView::Ram];
    if storage {
        views.push(ConfigurationView::Storage);
    }
    for view in views {
        let geometry = session.configuration_geometry(view).await?;
        let capacity = geometry.capacity;
        let page = geometry.page_bytes;
        // A read may span pages; the page size is the device's preferred
        // granularity, not the reply bound.
        let limit = NonZeroU32::new(chunk_bytes.max(page.get())).expect("chunk");
        let started = std::time::Instant::now();
        let mut bytes = Vec::with_capacity(capacity as usize);
        while (bytes.len() as u32) < capacity {
            let offset = bytes.len() as u32;
            let length = (capacity - offset).min(limit.get());
            let command = ReadConfiguration::new(
                ConfigurationLocation {
                    view,
                    page: 0,
                    offset,
                },
                NonZeroU32::new(length).ok_or("zero length")?,
                limit,
            )?;
            let chunk = session.read_configuration(command).await?;
            if chunk.len() != length as usize {
                return Err(format!("short read at {offset:#x}: {} of {length}", chunk.len()).into());
            }
            bytes.extend_from_slice(&chunk);
            if offset % 0x40000 == 0 {
                println!(
                    "  {view:?} {offset:#x}/{:x} in {:.1}s",
                    capacity,
                    started.elapsed().as_secs_f32()
                );
            }
        }
        let name = match view {
            ConfigurationView::Storage => "storage",
            ConfigurationView::Ram => "ram",
        };
        let path = out.join(format!("config-{name}.bin"));
        fs::write(&path, &bytes)?;
        let digest = Sha256::digest(&bytes);
        println!(
            "{view:?} capacity {capacity:#x} page {page} -> {} ({} bytes, sha256 {:x})",
            path.display(),
            bytes.len(),
            digest,
        );
    }
    Ok(())
}
