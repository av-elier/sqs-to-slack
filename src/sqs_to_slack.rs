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
        let source = sqs::SqsSource::new(&self.sqs_queue_name)?;
        loop {
            let msg = source.read()?;
            slack::send(&self.slack_hook_url, &msg)?;
        }
    }
}
