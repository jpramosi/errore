use std::time::Duration;

use errore::prelude::*;

pub mod database {
    use super::*;

    #[derive(Debug, Display)]
    pub enum DuplicateKind {
        Column,
        Cursor,
        Database,
        #[display("Duplicate function '{0}'")] // overrides "DuplicateKind::Function"
        Function(String),
        PreparedStatement,
        #[display("Duplicate schema")] // overrides "DuplicateKind::Schema"
        Schema,
        Table,
        Alias,
        Object,
    }

    #[derive(Debug, Display)]
    pub struct QueryContext {
        pub statement: String,
        pub duration: Duration,
    }

    #[derive(Debug, Display)]
    pub enum UndefinedKind {
        Column,
        Function,
        Table,
        Parameter,
        Object,
    }

    /// Database related errors.
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Configuration limit exceeded")]
        ConfigExceeded,
        #[error("Disk is full")]
        DiskFull,
        #[error("Deserialization failed")]
        Deserialize,
        #[error("Insufficient resources")]
        InsufficientResources,
        #[error("Maximum amount of memory reached (> {0})")]
        OutOfMemory(usize),
        #[error("Too many clients connected (> {0})")]
        TooManyConnections(usize),
        #[error(transparent)]
        Duplicate(DuplicateKind),
        #[error(transparent)]
        Syntax(QueryContext),
        #[error(transparent)]
        Undefined(UndefinedKind),
    }

    pub fn execute(_statement: String) -> Result<(), Ec> {
        err!(Error::TooManyConnections(10))
    }
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if let Err(ec) = database::execute("SELECT CustomerName, City FROM Customers;".into()) {
        // print formatted error chain
        println!("{}", ec.trace());
    }
}
