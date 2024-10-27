mod account;
mod auth;

use errore::prelude::*;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if let Err(ec) = account::login("root@errore.dev", "123") {
        // print formatted error chain
        println!("{}", ec.trace());

        // print trace records
        println!("\nTrace records:");
        for tr in &ec {
            println!("{}", tr);
        }

        // print the origin of the error
        // (the deepest 'Display' trait implementation will be used)
        println!("\nError display:\n{}", ec);

        // error extraction with 'match':
        // useful for handling multiple errors
        match ec.error() {
            account::Error::Authentication(ec) => match ec.error() {
                auth::Error::ReadPassword(error) => {
                    println!(
                        "\nError extraction with 'match':\nOS error code {}: {}",
                        error.raw_os_error().unwrap_or_default(),
                        error.kind()
                    )
                }
                _ => {}
            },
            _ => {}
        }

        // error extraction with 'get()':
        // useful for deeply nested errors
        if let Some(auth_error) = ec.get::<auth::Error>() {
            match &*auth_error {
                auth::Error::ReadPassword(error) => println!(
                    "\nError extraction with 'get()':\nOS error code {}: {}",
                    error.raw_os_error().unwrap_or_default(),
                    error.kind()
                ),
                _ => {}
            }
        }
    }
}
