#[macro_use]
extern crate log;
extern crate config;
extern crate env_logger;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate serde_json;
extern crate rusoto_core;
extern crate rusoto_sqs;

mod slack;
mod sqs;

use std::error::Error;

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
    println!("Hello, world!");

    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("settings")).unwrap();

    let sqs_queue_name = settings.get_str("sqs_queue_name")?;
    let slack_url = settings.get_str("slack_hook_url")?;

    loop {
        let msg = sqs::read(&sqs_queue_name)?;
        slack::send(&slack_url, &msg)?;
    }
}
