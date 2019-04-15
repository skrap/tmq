extern crate futures;
extern crate pretty_env_logger;
extern crate tmq;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate failure;

use futures::{Future, Sink, Stream};

use failure::Error;

use tokio::timer::Interval;

use tmq::*;

use std::env;
use std::time::Duration;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "publish=DEBUG");
    }

    pretty_env_logger::init();

    let request = publish(&Context::new())
        .bind("tcp://127.0.0.1:7899")
        .expect("Couldn't bind")
        .finish()
        .send_all(make_broadcast())
        .map(|_| ())
        .map_err(|e| {
            error!("Error publishing:{}", e);
        });

    tokio::run(request);
}

//Set up a timer to broadcast every second.
fn make_broadcast() -> impl Stream<Item = Message, Error = Error> {
    let mut i = 0;

    Interval::new_interval(Duration::from_millis(1000))
        .map(move |_| {
            i += 1;
            let message = format!("Broadcast #{}", i);
            info!("Publish: {}", message);
            Message::from(&message)
        })
        .from_err()
}
