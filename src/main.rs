#[macro_use]
extern crate log;
extern crate config;
extern crate env_logger;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rusoto_core;
extern crate rusoto_sqs;
extern crate tokio;

mod slack;
mod sqs;
mod sqs_to_slack;

use std::error::Error;
use std::thread;

fn main() {
    info!("started sqs-to-slack");
    env_logger::init();
    let result = main_result();
    info!("finished sqs-to-slack");
    match result {
        Ok(_) => {
            info!("success");
            std::process::exit(0)
        }
        Err(e) => {
            error!("error {:?}", e);
            std::process::exit(1)
        }
    }
}

fn main_result() -> Result<(), Box<Error>> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("settings")).unwrap();
    #[derive(Deserialize)]
    struct Connectors {
        connectors: Vec<sqs_to_slack::SqsToSlack>,
    };
    let connectors: Connectors = settings.try_into().unwrap();
    let mut handles = Vec::with_capacity(connectors.connectors.len());
    for connector in connectors.connectors {
        handles.push(thread::spawn(move || {
            connector.run().unwrap();
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}
