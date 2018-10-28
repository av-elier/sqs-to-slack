use slack;
use sqs;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct SqsToSlack {
    sqs_queue_name: String,
    slack_hook_url: String,
}

impl SqsToSlack {
    pub fn run(&self) -> Result<(), Box<Error>> {
        let rt = new_runtime();
        let source = sqs::SqsSource::new(&self.sqs_queue_name)?;
        let mut sender = slack::SlackSenderHttps::new(&self.slack_hook_url, rt)?;
        loop {
            let msg = source.read()?;
            sender.send(&msg)?;
        }
    }
}

fn new_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new()
        .core_threads(2)
        .blocking_threads(2)
        .build()
        .expect("runtime build")
}
