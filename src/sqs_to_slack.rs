use slack;
use sqs;
use std::error::Error;
// use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SqsToSlack {
    sqs_queue_name: String,
    slack_hook_url: String,
}

impl SqsToSlack {
    pub fn run(&self, executor: tokio::runtime::TaskExecutor) -> Result<(), Box<Error>> {
        let source = sqs::SqsSource::new(&self.sqs_queue_name)?;
        let mut sender = slack::SlackSenderHttps::new(&self.slack_hook_url, executor)?;
        loop {
            let msg = source.read()?;
            sender.send(&msg)?;
        }
    }
}

pub fn new_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new()
        .core_threads(2)
        .blocking_threads(2)
        .build()
        .expect("runtime build")
}
