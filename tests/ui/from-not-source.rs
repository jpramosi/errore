use errore::*;

#[derive(Debug, Error)]
pub struct Error {
    #[source]
    source: std::io::Error,
    #[from]
    other: anyhow::Error,
}

fn main() {}
