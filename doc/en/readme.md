### Use

`expire_map` : lock-free dictionary supporting a maximum of 256 cycles timeout (internally implemented using dashmap).

Also, I implement RetryMap based on ExpireMap and can be used for network request timeouts and retries.

### RetryMap usage demo

> ~/examples/main.rs

### ExpireMap usage demo

The use of ExpireMap can be seen in the RetryMap implementation

> ~/src/retry.rs

### About

This project is part of **[rmw.link](//rmw.link)** Code Project

![rmw.link logo](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)
