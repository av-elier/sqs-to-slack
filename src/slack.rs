use std::error::Error;

use hyper::rt::{self, Future, Stream};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;

use http::header::HeaderValue;

use std::thread;

pub fn send(url: &str, msg: &str) -> Result<(), Box<Error>> {
    info!("sending to slack ({}) '{}'", url, msg);

    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri: hyper::Uri = url.parse()?;

    let body_json = json!({
        "text": msg,
    });

    let mut req = Request::new(Body::from(body_json.to_string()));
    *req.method_mut() = Method::POST;
    *req.uri_mut() = uri.clone();
    req.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let res = client
        .request(req)
        .map(|res| {
            println!("Response code: {}", res.status());
            println!("Headers: {:#?}", res.headers());

            let body = res.into_body().concat2();
            println!("Response: {:?}", body.wait());
        })
        .map_err(|err| {
            println!("Error: {}", err);
        });

    thread::spawn(|| {
        rt::run(res);
        Ok::<(), ()>(())
    });

    Ok(())
}
