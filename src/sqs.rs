use std::error::Error;

use rusoto_core::Region;
use rusoto_sqs::{GetQueueUrlRequest, ReceiveMessageRequest, Sqs, SqsClient};

pub fn read(queue_name: &str) -> Result<String, Box<Error>> {
    let client = SqsClient::new(Region::UsEast2);

    let mut req_queue_url = GetQueueUrlRequest::default();
    req_queue_url.queue_name = queue_name.to_string();
    let queue_url_result = client.get_queue_url(req_queue_url).sync()?;
    let queue_url = queue_url_result.queue_url.unwrap();

    let mut req_recieve = ReceiveMessageRequest::default();
    req_recieve.queue_url = queue_url;
    req_recieve.max_number_of_messages = Some(1);

    loop {
        let recieve_result = client.receive_message(req_recieve.clone()).sync()?;

        let msg = match &recieve_result.messages {
            None => {
                info!("recieved no messages yet");
                continue;
            }
            Some(msgs) => &msgs[0],
        };

        info!("recieved sqs message {:?}", msg);

        return Ok(format!("{:?}", msg));
    }
}
