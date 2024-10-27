use errore::*;

#[derive(Error, Debug)]
#[error(transparent)]
#[error("...")]
pub struct Error(anyhow::Error);

fn main() {}
