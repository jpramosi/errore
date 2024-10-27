use errore::*;

#[derive(Debug)]
struct NotError;

#[derive(Error, Debug)]
#[error("...")]
pub struct ErrorStruct {
    source: NotError,
}

fn main() {}
