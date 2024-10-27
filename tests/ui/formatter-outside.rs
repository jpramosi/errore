use errore::formatter::Formatter;

#[derive(Clone, Default, Debug)]
pub struct MyFormatter;

impl Formatter for MyFormatter {}

errore::formatter!(MyFormatter);

fn main() {}
