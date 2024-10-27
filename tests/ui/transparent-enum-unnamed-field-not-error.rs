use errore::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Other(String),
}

fn main() {}
