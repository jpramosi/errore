use errore::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Other { message: String },
}

fn main() {}
