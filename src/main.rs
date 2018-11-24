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
extern crate rusoto_credential;
extern crate rusoto_sqs;
extern crate rusoto_sts;
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

#[derive(Deserialize)]
struct Settings {
    connectors: Vec<sqs_to_slack::SqsToSlack>,

    dns_worker_threads: usize,
    tokio_core_threads: usize,
}

fn main_result() -> Result<(), Box<Error>> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("settings")).unwrap();
    let sett: Settings = settings.try_into().unwrap();

    let rt = tokio::runtime::Builder::new()
        .core_threads(sett.tokio_core_threads)
        .build()
        .expect("tokio runtime initialization failed");

    let https =
        hyper_tls::HttpsConnector::new(sett.dns_worker_threads).expect("TLS initialization failed");
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let mut handles = Vec::with_capacity(sett.connectors.len());
    for connector in sett.connectors {
        let executor = rt.executor();
        let client = client.clone();
        handles.push(thread::spawn(move || {
            connector
                .run(executor, client)
                .expect("failed to start sqs-rust connector");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
