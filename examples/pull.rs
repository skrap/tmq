extern crate futures;
extern crate pretty_env_logger;
extern crate tmq;
extern crate tokio;

#[macro_use]
extern crate log;

extern crate failure;

use futures::{Future, Stream};

use tmq::*;

use std::env;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "pull=DEBUG");
    }

    pretty_env_logger::init();

    let request = pull(&Context::new())
        .bind("tcp://127.0.0.1:7899")
        .expect("Couldn't bind")
        .finish()
        .for_each(|parts| {
            info!("Pull: {}", parts.iter().map(|m| m.as_str().unwrap_or("")).join(" "));
            Ok(())
        })
        .map_err(|e| {
            error!("Error Pulling: {}", e);
        });

    tokio::run(request);
}
