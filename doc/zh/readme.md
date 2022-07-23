`expire_map` : 最大支持 256 个周期超时的高并发字典（内部使用 dashmap 实现）。

和现有的 rust 超时字典不同，在超时回调的参数中有上下文对象，这样可以避免在每个超时对象中浪费内存空间放置上下文指针。

同时，我还基于 ExpireMap 实现了 RetryMap，可以用于网络请求超时和重试。

## RetryMap 使用演示

> ~/examples/main.rs

运行输出

> ~/out.txt

## ExpireMap 使用演示

ExpireMap 的使用可以参见 RetryMap 的实现

> ~/src/retry.rs
