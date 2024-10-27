mod task;

use errore::prelude::*;

pub mod codec {
    use super::*;

    /// Codec related errors.
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Corrupted data at position {pos}")]
        Corrupted { pos: usize },
        // Use error context (Ec) from the wrapper.
        #[error(transparent)]
        Tokio(#[from] task::Ec),
    }

    async fn decode_background(_buf: Vec<u8>) -> Result<(), Ec> {
        Err(Ec::new(Error::Corrupted { pos: 0 }))
    }

    pub async fn decode(buf: Vec<u8>) -> Result<(), Ec> {
        // Use custom tokio task wrapper for converting the std result to errore result.
        let task = task::spawn(decode_background(buf)).await;

        match task {
            // handle decode error
            Ok(r) => r?,
            // handle JoinError
            Err(err) => {
                eprintln!("Task failed to run {}", err)
            }
        }

        Ok(())
    }
}

pub mod net {
    use super::*;

    /// Download related errors.
    #[derive(Error, Debug)]
    pub enum Error {
        #[error(transparent)]
        IO(#[from] std::io::Error),
        // Use tokio`s error instead of the wrapper type.
        #[error(transparent)]
        Tokio(#[from] tokio::task::JoinError),
    }

    async fn download_from(_url: &str) -> Result<(), Ec> {
        err!(Error::IO(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused"
        )))
    }

    pub async fn download(urls: &'static [&'static str]) -> Vec<(Result<(), Ec>, String)> {
        // Errore can be also used without any wrapper.
        // But if the result is handled explicitly, it needs the fully qualified path.
        let mut set = tokio::task::JoinSet::new();

        for url in urls {
            set.spawn(async {
                let url = url.to_string();
                (download_from(&url).await, url)
            });
        }

        set.join_all().await
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = codec::decode(vec![0x47, 0x21, 0x45]).await {
        println!("Failed to decode buffer: {}", err.trace());
    }

    for r in net::download(&[
        "https://www.rust-lang.org/static/images/cli.svg",
        "https://www.rust-lang.org/static/images/webassembly.svg",
    ])
    .await
    {
        if let Err(err) = r.0 {
            println!("Failed to download '{}' {}", r.1, err.trace());
        }
    }
}
