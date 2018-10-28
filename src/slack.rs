use std::error::Error;

use hyper::rt::{Future, Stream};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use http::header::HeaderValue;

pub struct SlackSender<T> {
    rt: tokio::runtime::Runtime,
    client: hyper::Client<T>,
    hook_uri: hyper::Uri,
}

type Https = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

pub type SlackSenderHttps = SlackSender<Https>;

impl SlackSender<Https> {
    pub fn new(
        hook_url: &str,
        rt: tokio::runtime::Runtime,
    ) -> Result<SlackSender<Https>, Box<Error>> {
        let https = HttpsConnector::new(4).expect("TLS initialization failed");
        let client = Client::builder().build::<_, hyper::Body>(https);
        let uri: hyper::Uri = hook_url.parse()?;

        Ok(SlackSender {
            rt,
            client: client,
            hook_uri: uri,
        })
    }

    pub fn send(&mut self, msg: &str) -> Result<(), Box<Error>> {
        let body_json = json!({
            "text": msg,
        });

        let mut req = Request::new(Body::from(body_json.to_string()));
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
        self.rt.spawn(res);
        Ok(())
    }
}
