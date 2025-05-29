# download-extract-progress

A Rust library for downloading and extracting files with progress tracking. Supports async downloads, hash verification, and zip extraction with progress updates.

---

## Features

- **Async file download** with progress reporting
- **SHA256 hash verification**
- **zip archive extraction** with progress
- **GitHub release asset download**
- **Error handling** with rich error types

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
download-extract-progress = "0.1.0"
```

---

## Usage

### Download a file with progress

```rust
use download_extract_progress::download::download;
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let url = "https://example.com/file.zip";
    let path = "./file.zip";
    let mut stream = download("Example File", url, path, None).await;
    while let Some(status) = stream.next().await {
        match status {
            Ok((progress, msg)) => println!("{:.0}% - {}", progress * 100.0, msg),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

### Extract a zip archive with progress

```rust
use download_extract_progress::extract::extract_obs;
use futures_util::StreamExt;
use std::path::Path;

#[tokio::main]
async fn main() {
    let archive = Path::new("./file.zip");
    let out_dir = Path::new("./output");
    let mut stream = extract_obs(archive, out_dir).await;
    while let Some(status) = stream.next().await {
        match status {
            Ok((progress, msg)) => println!("{:.0}% - {}", progress * 100.0, msg),
            Err(e) => eprintln!("Extract error: {}", e),
        }
    }
}
```

### Download the latest GitHub release asset

```rust
use download_extract_progress::download::download_github;
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let repo = "user/repo";
    let is_zip = |name: &str| name.ends_with(".zip");
    let path = "./latest.zip";
    let mut stream = download_github(repo, is_zip, path, None).await.unwrap();
    while let Some(status) = stream.next().await {
        match status {
            Ok((progress, msg)) => println!("{:.0}% - {}", progress * 100.0, msg),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

---

## License

MIT

---

## Contributing

Pull requests and issues are welcome!

---

## Credits

- [reqwest](https://github.com/seanmonstar/reqwest)
- [tokio](https://tokio.rs/)
- [sevenz-rust](https://github.com/sile/sevenz-rust)

---

## API Docs

See [docs.rs](https://docs.rs/download-extract-progress) for full API documentation.
