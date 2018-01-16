Examples
========

* [echo-pair.rs](echo-pair.rs) - Sending and receiving simple messages with futures.

  A PAIR of sockets is created. The `sender` socket sends
  a message, and the `receiver` gets it.

* [echo-push-pull-multipart](echo-push-pull-multipart.rs) - Sending and receiving multi-part messages with futures.

  This time we use `PUSH` and `PULL` sockets to move multi-part messages.

  Remember that ZMQ will either send all parts or none at all.
  Save goes for receiving.


* [echo-pub-sub](echo-pub-sub.rs) - Manual use of tokio tranports with `Sink` and `Stream`

  This time, we use `PUB`-`SUB` sockets to send and receive a message.
