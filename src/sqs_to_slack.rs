use slack;
use sqs;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct SqsToSlack {
    sqs_queue_name: String,
    slack_hook_url: String,
}

impl SqsToSlack {
    pub fn run(
        &self,
        executor: tokio::runtime::TaskExecutor,
        client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    ) -> Result<(), Box<Error>> {
        let source = sqs::SqsSource::new(&self.sqs_queue_name)?;

        let mut sender = slack::SlackSenderHttps::new(&self.slack_hook_url, executor, client)?;

        loop {
            let msg = source.read()?;
            sender.send(&msg)?;
        }
    }
}
