use std::path::Path;

use async_stream::stream;
use futures_core::Stream;
use sevenz_rust::{default_entry_extract_fn, Password, SevenZReader};

/// Extracts a zip archive to the given output path, reporting progress as a stream.
///
/// # Arguments
/// * `file` - Path to the zip archive.
/// * `out_path` - Output directory for extraction.
///
/// # Returns
/// A stream yielding progress (0.0-1.0) and status messages, or errors.
pub async fn extract(file: &Path, out_path: &Path) -> impl Stream<Item = Result<(f32, String), sevenz_rust::Error>>{
    let dest = out_path.to_path_buf();
    let path = file.to_path_buf();

    stream! {
        yield Ok((0.0, "Reading file...".to_string()));
        let mut sz = SevenZReader::open(&path, Password::empty())?;
        let (tx, mut rx) = tokio::sync::mpsc::channel(5);

        let total = sz.archive().files.len() as f32;
        if !dest.exists() {
            std::fs::create_dir_all(&dest)?;
        }

        let mut curr = 0;
        let mut r = tokio::task::spawn_blocking(move || {
            sz.for_each_entries(|entry, reader| {
                curr += 1;
                tx.blocking_send((curr as f32 / total, format!("Extracting {}", entry.name()))).unwrap();

                let dest_path = dest.join(entry.name());

                default_entry_extract_fn(entry, reader, &dest_path)
            })?;

            Result::<_, sevenz_rust::Error>::Ok((1.0, "Extraction done".to_string()))
        });

        loop {
            tokio::select! {
                m = rx.recv() => {
                    match m {
                        Some(e) => yield Ok(e),
                        None => break
                    }
                },
                res = &mut r => {
                    let res = res.unwrap();
                    yield res;

                    break;
                }
            };
        }

        yield Ok((1.0, "Extraction done".to_string()));
    }
}