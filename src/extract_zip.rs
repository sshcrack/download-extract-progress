use std::path::Path;

use async_stream::stream;
use futures_core::Stream;
use std::fs::{self, File};
use std::io;
use zip::read::ZipArchive;

/// Extracts a zip archive to the given output path, reporting progress as a stream.
///
/// # Arguments
/// * `file` - Path to the zip archive.
/// * `out_path` - Output directory for extraction.
///
/// # Returns
/// A stream yielding progress (0.0-1.0) and status messages, or errors.
pub async fn extract_zip(
    file: &Path,
    out_path: &Path,
) -> impl Stream<Item = Result<(f32, String), io::Error>> {
    let dest = out_path.to_path_buf();
    let path = file.to_path_buf();

    stream! {
        yield Ok((0.0, "Reading zip file...".to_string()));

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                yield Err(e);
                return;
            }
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(archive) => archive,
            Err(e) => {
                yield Err(io::Error::new(io::ErrorKind::Other, e));
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(5);

        let total = archive.len() as f32;
        if !dest.exists() {
            if let Err(e) = fs::create_dir_all(&dest) {
                yield Err(e);
                return;
            }
        }

        let mut r = tokio::task::spawn_blocking(move || {
            for i in 0..archive.len() {
                let mut file = match archive.by_index(i) {
                    Ok(file) => file,
                    Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
                };

                let file_name = match file.enclosed_name() {
                    Some(path) => path.to_owned(),
                    None => continue, // Skip files with unsafe paths
                };

                let progress = (i as f32 + 1.0) / total;
                let _ = tx.blocking_send((progress, format!("Extracting {}", file_name.display())));

                let out_path = dest.join(file_name);

                if file.is_dir() {
                    fs::create_dir_all(&out_path)?;
                } else {
                    if let Some(parent) = out_path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }

                    let mut out_file = fs::File::create(&out_path)?;
                    io::copy(&mut file, &mut out_file)?;
                }
            }

            Ok((1.0, "Extraction done".to_string()))
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
                    match res {
                        Ok(res) => {
                            match res {
                                Ok(val) => yield Ok(val),
                                Err(e) => yield Err(e),
                            }
                        },
                        Err(e) => yield Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
                    }
                    break;
                }
            };
        }

        yield Ok((1.0, "Extraction complete".to_string()));
    }
}
