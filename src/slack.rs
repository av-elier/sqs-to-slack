use std::error::Error;

use http::header::HeaderValue;
use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use sqs;

pub struct SlackSender<T> {
    executor: tokio::runtime::TaskExecutor,
    client: hyper::Client<T>,
    hook_uri: hyper::Uri,
}

type Https = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

pub type SlackSenderHttps = SlackSender<Https>;

impl SlackSender<Https> {
    pub fn new(
        hook_url: &str,
        executor: tokio::runtime::TaskExecutor,
    ) -> Result<SlackSender<Https>, Box<Error>> {
        let https = HttpsConnector::new(4).expect("TLS initialization failed");
        let client = Client::builder().build::<_, hyper::Body>(https);
        let uri: hyper::Uri = hook_url.parse()?;

        Ok(SlackSender {
            executor,
            client: client,
            hook_uri: uri,
        })
    }

    pub fn send(&mut self, msg: &sqs::Message) -> Result<(), Box<Error>> {
        let body = format_sqs_message(msg);

        let mut req = Request::new(Body::from(body));
        *req.method_mut() = Method::POST;
        *req.uri_mut() = self.hook_uri.clone();
        req.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let res = self
            .client
            .request(req)
            .map(|res| {
                debug!("Response code: {}", res.status());
                debug!("Headers: {:#?}", res.headers());

                let body = res.into_body().concat2();
                debug!("Response: {:?}", body.wait());
            }).map_err(|err| {
                error!("Error: {}", err);
            });
        self.executor.spawn(res);
        Ok(())
    }
}

fn format_sqs_message(msg: &sqs::Message) -> String {
    let sqs_msg = msg.sqs_msg.clone();
    let msg_json = json!({
        "text": format!("Got message from queue *{}*", msg.queue_name),
        "attachments": [
            {
                "title": format!("Id {}", sqs_msg.clone().message_id.unwrap_or("_no id_".to_string())),
                "text": format!("{:?}", sqs_msg.clone().attributes.unwrap_or(std::collections::HashMap::new())),
            },
            {
                "title": "The message's contents",
                "text": format!("```{}```", sqs_msg.clone().body.unwrap_or("_no body_".to_string())),
            },
        ],
    });
    msg_json.to_string()
}
