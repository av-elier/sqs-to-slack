
Intended to work on dead-letter queue, where human actions needed to be taken on each dead message.

# TODO
- once create `hyper::Client`, and once spawn thread with `hyper::rt::run`,
- report not only sqs message content but also queue name, and attributes,
- make "sqs queue -> slack channel" pairs configurable,
- make report message more beautiful,
- use tokio-core reactor for sqs instead of sync,
- make async stack send,
- remove message after sending to slack.
