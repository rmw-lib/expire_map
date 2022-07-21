### Use

`expire_map` : High concurrency dictionary supporting a maximum of 256 cycles timeout (internally implemented using dashmap).

Also, I implement RetryMap based on ExpireMap and can be used for network request timeouts and retries.

### RetryMap usage demo

> ~/examples/main.rs

Output

> ~/out.txt

### ExpireMap usage demo

The use of ExpireMap can be seen in the RetryMap implementation

> ~/src/retry.rs
