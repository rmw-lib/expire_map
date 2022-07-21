<!-- EDIT /Users/z/rmw/expire_map/README.md -->

<h1 align="center"> expire_map</h1>
<p align="center">
<a href="#english-readme">English</a>
|
<a href="#中文说明 "> 中文说明 </a>
</p>

---

## English Readme

<!-- EDIT /Users/z/rmw/expire_map/doc/en/readme.md -->

### Use

[→ examples/main.rs](examples/main.rs)

```rust
use std::{net::SocketAddrV4, time::Duration};

use anyhow::Result;
use async_std::task::{block_on, sleep, spawn};
use expire_map::{Caller, RetryMap};

#[derive(Debug)]
struct Task {
  addr: SocketAddrV4,
  msg: Box<[u8]>,
}

impl Caller for Task {
  fn ttl() -> u8 {
    2
  }
  fn call(&self) {
    dbg!(self);
  }
  fn fail(&self) {
    dbg!(("failed", self));
  }
}

fn main() -> Result<()> {
  let task = Task {
    addr: "223.5.5.5:53".parse()?,
    msg: Box::from(&[1, 2, 3][..]),
  };

  let retry_times = 3;
  let task_id = 1;

  let retry_map = RetryMap::new();

  let expireer = retry_map.expire.clone();

  let handle = spawn(async move {
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      dbg!("do expire");
    }
  });

  retry_map.insert(task_id, task, retry_times);
  dbg!(retry_map.get(&task_id).unwrap().value());

  block_on(handle);
  Ok(())
}
```


### Link

* [benchmark report log](https://rmw-lib.github.io/expire_map/dev/bench/)

### About

This project is part of **[rmw.link](//rmw.link)** Code Project

![rmw.link logo](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)

---

## 中文说明

<!-- EDIT /Users/z/rmw/expire_map/doc/zh/readme.md -->

expire_map : 最大支持 256 个周期超时的无锁字典。

用于网络请求超时和重试。

### 使用

[→ examples/main.rs](examples/main.rs)

```rust
use std::{net::SocketAddrV4, time::Duration};

use anyhow::Result;
use async_std::task::{block_on, sleep, spawn};
use expire_map::{Caller, RetryMap};

#[derive(Debug)]
struct Task {
  addr: SocketAddrV4,
  msg: Box<[u8]>,
}

impl Caller for Task {
  fn ttl() -> u8 {
    2
  }
  fn call(&self) {
    dbg!(self);
  }
  fn fail(&self) {
    dbg!(("failed", self));
  }
}

fn main() -> Result<()> {
  let task = Task {
    addr: "223.5.5.5:53".parse()?,
    msg: Box::from(&[1, 2, 3][..]),
  };

  let retry_times = 3;
  let task_id = 1;

  let retry_map = RetryMap::new();

  let expireer = retry_map.expire.clone();

  let handle = spawn(async move {
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      dbg!("do expire");
    }
  });

  retry_map.insert(task_id, task, retry_times);
  dbg!(retry_map.get(&task_id).unwrap().value());

  block_on(handle);
  Ok(())
}
```


### 相关连接

* [性能评测日志](https://rmw-lib.github.io/expire_map/dev/bench/)

### 关于

本项目隶属于 **人民网络 ([rmw.link](//rmw.link))** 代码计划。

![人民网络海报](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)
