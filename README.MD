
Intended to work on dead-letter queue, where human actions needed to be taken on each dead message.

# TODO
- once create `hyper::Client`, and once spawn thread with `hyper::rt::run`,
- make "sqs queue -> slack channel" pairs configurable,
- use tokio-core reactor for sqs instead of sync,
- make async stack send,
- remove message after sending to slack.
