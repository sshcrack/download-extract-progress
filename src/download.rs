use std::path::Path;

use anyhow::Context;
use async_stream::stream;
use futures_core::Stream;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{error::DownloadError, github_releases};

/// Downloads a file from the given URL to the specified path, reporting progress as a stream.
///
/// # Arguments
/// * `display_name` - A name for display in progress messages.
/// * `url` - The URL to download from.
/// * `path` - The destination file path.
/// * `expected_hash` - Optional SHA256 hash to verify the download.
///
/// # Returns
/// A stream yielding progress (0.0-1.0) and status messages, or errors.
pub async fn download<T: AsRef<Path>>(
    display_name: &str,
    url: &str,
    path: T,
    expected_hash: Option<String>,
) -> impl Stream<Item = Result<(f32, String), DownloadError>> + use<T> {
    let display_name = display_name.to_string();
    let url = url.to_string();

    let client = reqwest::Client::new();
    stream! {
        yield Ok((0.0, format!("Downloading {display_name}")));

        let res = client.get(url).send().await;
        if let Err(e) = res {
            yield Err(DownloadError::RequestError(e));
            return;
        }

        let res = res.unwrap();
        let length = res.content_length().unwrap_or(0);

        let mut bytes_stream = res.bytes_stream();

        let tmp_file = File::create_new(&path)
            .await
            .map_err(|e| DownloadError::IoError(e));

        if let Err(e) = tmp_file {
            yield Err(e);
            return;
        }

        let mut curr_len = 0;
        let mut hasher = Sha256::new();

        let mut tmp_file = tmp_file.unwrap();
        while let Some(chunk) = bytes_stream.next().await {
            if let Err(e) = chunk {
                yield Err(DownloadError::RequestError(e));
                return;
            }

            let chunk = chunk.unwrap();
            hasher.update(&chunk);
            let r = tmp_file.write_all(&chunk).await;
            if let Err(e) = r {
                yield Err(DownloadError::IoError(e));
                return;
            }

            curr_len = std::cmp::min(curr_len + chunk.len() as u64, length);
            yield Ok((curr_len as  f32 / length as f32, format!("Downloading {display_name}")));
        }

        if let Some(expected_hash) = expected_hash {
            let remote_hash = hex::decode(expected_hash);
            if let Err(e) = remote_hash {
                yield Err(DownloadError::InvalidHash(e));
                return;
            }

            let remote_hash = remote_hash.unwrap();

            // Calculating local hash
            let local_hash = hasher.finalize();
            if local_hash.as_slice() != remote_hash {
                let remote_hash = hex::encode(remote_hash);
                let local_hash = hex::encode(local_hash);

                yield Err(DownloadError::HashMismatch(remote_hash, local_hash));
                return;
            }

            log::trace!("Hashes match");
        }
    }
}

/// Downloads the latest GitHub release asset matching a predicate, with optional hash verification.
///
/// # Arguments
/// * `repo` - GitHub repo in `owner/name` format.
/// * `is_valid_file` - Predicate to select the asset.
/// * `path` - Destination file path.
/// * `hash_url` - Optional URL to a hash file.
///
/// # Returns
/// A stream yielding progress and status, or errors.
pub async fn download_github<T: AsRef<Path>, K>(
    repo: &str,
    is_valid_file: K,
    path: T,
    hash_url: Option<String>,
) -> anyhow::Result<impl Stream<Item = Result<(f32, String), DownloadError>>>
where
    K: Fn(&str) -> bool,
{
    let display_name = repo.split('/').last().unwrap_or("unknown");

    let client = reqwest::Client::builder()
        .user_agent("github-releases-downloader/1.0")
        .build()
        .context("Creating HTTP client")?;

    let releases: github_releases::Root = client
        .get(format!("https://api.github.com/repos/{repo}/releases"))
        .send().await?
        .json().await?;

    let latest_version = releases
        .iter()
        .max_by_key(|r| &r.published_at)
        .context("Finding latest version")?;

    let archive_url = latest_version
        .assets
        .iter()
        .find(|a| is_valid_file(&a.name))
        .context("Finding zip asset")?
        .browser_download_url
        .clone();

    let display_name = display_name.to_string();
    let expected_hash = if let Some(url) = hash_url {
        let hash: String = reqwest::get(url).await?.text().await?;
        Some(hash.trim().to_string())
    } else {
        None
    };

    Ok(download(&display_name, &archive_url, path, expected_hash).await)
}
