use std::error::Error;

use rusoto_core::Region;
use rusoto_sqs::{DeleteMessageRequest, GetQueueUrlRequest, ReceiveMessageRequest, Sqs, SqsClient};
use std::sync::Arc;

pub struct SqsSource {
    client: Arc<Sqs>,
    queue_name: String,
    queue_url: String,
}

impl SqsSource {
    pub fn new(queue_name: &str) -> Result<SqsSource, Box<Error>> {
        let client = SqsClient::new(Region::UsEast2);

        let mut req_queue_url = GetQueueUrlRequest::default();
        req_queue_url.queue_name = queue_name.to_string();
        let queue_url_result = client.get_queue_url(req_queue_url).sync()?;
        let queue_url = queue_url_result.queue_url.unwrap();
        Ok(SqsSource {
            client: Arc::new(client),
            queue_name: queue_name.to_string(),
            queue_url: queue_url,
        })
    }

    pub fn read(&self, delete_on_read: bool) -> Result<Message, Box<Error>> {
        let mut req_recieve = ReceiveMessageRequest::default();
        req_recieve.queue_url = self.queue_url.to_string();
        req_recieve.max_number_of_messages = Some(1);
        req_recieve.attribute_names = Some(vec![String::from("All")]);
        req_recieve.wait_time_seconds = Some(1);

        loop {
            let recieve_result = self.client.receive_message(req_recieve.clone()).sync()?;

            let msg = match &recieve_result.messages {
                None => {
                    info!("recieved no messages yet");
                    continue;
                }
                Some(msgs) => &msgs[0],
            };

            info!("recieved sqs message ({}) {:?}", self.queue_name, msg);

            // this is bad, we could delete a message before successfull sending it to slack.
            if delete_on_read {
                info!("will try to delete message");
                let del = DeleteMessageRequest {
                    queue_url: self.queue_url.to_string(),
                    receipt_handle: msg.clone().receipt_handle.unwrap().clone(),
                };
                self.client.delete_message(del).sync()?;
            }

            return Ok(Message {
                queue_name: self.queue_name.clone(),
                sqs_msg: msg.clone(),
            });
        }
    }
}

pub struct Message {
    pub queue_name: String,
    pub sqs_msg: rusoto_sqs::Message,
}
