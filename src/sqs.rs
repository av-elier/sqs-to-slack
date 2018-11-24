use std::error::Error;

use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_sqs::{
    DeleteMessageRequest, GetQueueUrlRequest, ListQueuesRequest, ReceiveMessageRequest, Sqs,
    SqsClient,
};
use rusoto_sts::{GetSessionTokenRequest, Sts, StsClient};
use std::sync::Arc;

pub struct SqsSource {
    client: Arc<Sqs>,
    queue_name: String,
    queue_url: String,
}

impl SqsSource {
    pub fn new(queue_name: &str) -> Result<SqsSource, Box<Error>> {
        let token_client = StsClient::new(Region::UsEast2);
        let token_resp = token_client
            .get_session_token(GetSessionTokenRequest::default())
            .sync()?;
        info!("got token credentials are {:?}", token_resp);
        let creds = token_resp.credentials.expect("failed to get aws token");
        let dispatcher = HttpClient::new().expect("failed to create request dispatcher");
        let credentials = StaticProvider::new(
            creds.access_key_id,
            creds.secret_access_key,
            Some(creds.session_token),
            None,
        );

        let client = SqsClient::new_with(dispatcher, credentials, Region::UsEast1);

        info!("list sqs queues");
        let queues = client.list_queues(ListQueuesRequest::default()).sync()?;
        info!(
            "see sqs queues: {:?}",
            queues.queue_urls.expect("no queues available")
        );

        let mut req_queue_url = GetQueueUrlRequest::default();
        req_queue_url.queue_name = queue_name.to_string();
        let queue_url_result = client.get_queue_url(req_queue_url).sync()?;
        let queue_url = queue_url_result.queue_url.expect("failed to get queue url");
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
