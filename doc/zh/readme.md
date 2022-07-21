`expire_map` : 最大支持 256 个周期超时的无锁字典。

同时，基于 ExpireMap 实现了 RetryMap，可以用于网络请求超时和重试。

### RetryMap 使用演示

> ~/examples/main.rs

### ExpireMap 的使用可以参见 RetryMap 的实现

> ~/src/retry.rs

### 关于

本项目隶属于 **人民网络 ([rmw.link](//rmw.link))** 代码计划。

![人民网络海报](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)
