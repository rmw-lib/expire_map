## Use

`expire_map` : High concurrency timeout dictionary supporting a maximum of 256 cycles timeout (internally implemented using dashmap).

Unlike the existing rust expire map, there are context objects in the parameters of the timeout callback, which avoids wasting memory space for context pointers in each timeout object.

Also, I implement RetryMap based on ExpireMap and can be used for network request timeouts and retries.

## RetryMap usage demo

> ~/examples/main.rs

Output

> ~/out.txt

## ExpireMap usage demo

The use of ExpireMap can be seen in the RetryMap implementation

> ~/src/retry.rs
