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

`expire_map` : High concurrency dictionary supporting a maximum of 256 cycles timeout (internally implemented using dashmap).

Also, I implement RetryMap based on ExpireMap and can be used for network request timeouts and retries.

### RetryMap usage demo

[→ examples/main.rs](examples/main.rs)

```rust
use std::{net::SocketAddrV4, time::Duration};

use anyhow::Result;
use async_std::task::{block_on, sleep, spawn};
use expire_map::{Caller, RetryMap};

#[derive(Debug)]
struct Msg {
  msg: Box<[u8]>,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
struct Task {
  addr: SocketAddrV4,
  id: u16,
}

impl Caller<Task> for Msg {
  fn ttl() -> u8 {
    2 // 超时时间是2秒
  }
  fn call(&self, task: &Task) {
    dbg!(("call", task, &self.msg));
  }
  fn fail(&self, task: &Task) {
    dbg!(("failed", task, &self.msg));
  }
}

fn main() -> Result<()> {
  let msg = Msg {
    msg: Box::from(&[1, 2, 3][..]),
  };

  let task = Task {
    id: 12345,
    addr: "223.5.5.5:53".parse()?,
  };

  let retry_times = 3; // 重试次数是3次
  let retry_map = RetryMap::new();

  let expireer = retry_map.clone();

  let handle = spawn(async move {
    let mut do_expire = 0;
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      do_expire += 1;
      let exist = expireer.get(&task).is_some();
      dbg!((do_expire, exist));
    }
  });

  msg.call(&task);
  retry_map.insert(task, msg, retry_times);

  //dbg!(retry_map.get(&task).unwrap().value());
  //dbg!(retry_map.get_mut(&task).unwrap().key());
  //retry_map.remove(task);

  block_on(handle);
  Ok(())
}
```


### ExpireMap usage demo

The use of ExpireMap can be seen in the RetryMap implementation

[→ src/retry.rs](src/retry.rs)

```rust
use std::{fmt::Debug, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Caller<K> {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&self, key: &K);
  fn fail(&self, key: &K);
}

#[derive(Debug, Default)]
pub struct Retry<C> {
  n: u8,
  caller: C,
}

impl<K, C: Caller<K>> OnExpire<K> for Retry<C> {
  fn on_expire(&mut self, key: &K) -> u8 {
    self.caller.call(key);
    let n = self.n.wrapping_sub(1);
    if n == 0 {
      self.caller.fail(key);
      0
    } else {
      self.n = n;
      C::ttl()
    }
  }
}

#[derive(Debug, Default)]
pub struct RetryMap<K: Key, C: Caller<K> + Debug> {
  pub expire: ExpireMap<K, Retry<C>>,
}

impl<K: Key, C: Caller<K> + Debug> Clone for RetryMap<K, C> {
  fn clone(&self) -> Self {
    Self {
      expire: self.expire.clone(),
    }
  }
}

impl<K: Key, C: Caller<K> + Debug> RetryMap<K, C> {
  pub fn new() -> Self {
    Self {
      expire: ExpireMap::new(),
    }
  }

  pub fn insert(&self, key: K, caller: C, retry: u8) {
    self
      .expire
      .insert(key, Retry { n: retry, caller }, C::ttl());
  }
}

impl<K: Key, C: Caller<K> + Debug> Deref for RetryMap<K, C> {
  type Target = ExpireMap<K, Retry<C>>;
  fn deref(&self) -> &<Self as Deref>::Target {
    &self.expire
  }
}
```


### About

This project is part of **[rmw.link](//rmw.link)** Code Project

![rmw.link logo](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)

---

## 中文说明

<!-- EDIT /Users/z/rmw/expire_map/doc/zh/readme.md -->

`expire_map` : 最大支持 256 个周期超时的高并发字典（内部使用 dashmap 实现）。

同时，基于 ExpireMap 实现了 RetryMap，可以用于网络请求超时和重试。

### RetryMap 使用演示

[→ examples/main.rs](examples/main.rs)

```rust
use std::{net::SocketAddrV4, time::Duration};

use anyhow::Result;
use async_std::task::{block_on, sleep, spawn};
use expire_map::{Caller, RetryMap};

#[derive(Debug)]
struct Msg {
  msg: Box<[u8]>,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
struct Task {
  addr: SocketAddrV4,
  id: u16,
}

impl Caller<Task> for Msg {
  fn ttl() -> u8 {
    2 // 超时时间是2秒
  }
  fn call(&self, task: &Task) {
    dbg!(("call", task, &self.msg));
  }
  fn fail(&self, task: &Task) {
    dbg!(("failed", task, &self.msg));
  }
}

fn main() -> Result<()> {
  let msg = Msg {
    msg: Box::from(&[1, 2, 3][..]),
  };

  let task = Task {
    id: 12345,
    addr: "223.5.5.5:53".parse()?,
  };

  let retry_times = 3; // 重试次数是3次
  let retry_map = RetryMap::new();

  let expireer = retry_map.clone();

  let handle = spawn(async move {
    let mut do_expire = 0;
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      do_expire += 1;
      let exist = expireer.get(&task).is_some();
      dbg!((do_expire, exist));
    }
  });

  msg.call(&task);
  retry_map.insert(task, msg, retry_times);

  //dbg!(retry_map.get(&task).unwrap().value());
  //dbg!(retry_map.get_mut(&task).unwrap().key());
  //retry_map.remove(task);

  block_on(handle);
  Ok(())
}
```


### ExpireMap 使用演示

ExpireMap 的使用可以参见 RetryMap 的实现

[→ src/retry.rs](src/retry.rs)

```rust
use std::{fmt::Debug, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Caller<K> {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&self, key: &K);
  fn fail(&self, key: &K);
}

#[derive(Debug, Default)]
pub struct Retry<C> {
  n: u8,
  caller: C,
}

impl<K, C: Caller<K>> OnExpire<K> for Retry<C> {
  fn on_expire(&mut self, key: &K) -> u8 {
    self.caller.call(key);
    let n = self.n.wrapping_sub(1);
    if n == 0 {
      self.caller.fail(key);
      0
    } else {
      self.n = n;
      C::ttl()
    }
  }
}

#[derive(Debug, Default)]
pub struct RetryMap<K: Key, C: Caller<K> + Debug> {
  pub expire: ExpireMap<K, Retry<C>>,
}

impl<K: Key, C: Caller<K> + Debug> Clone for RetryMap<K, C> {
  fn clone(&self) -> Self {
    Self {
      expire: self.expire.clone(),
    }
  }
}

impl<K: Key, C: Caller<K> + Debug> RetryMap<K, C> {
  pub fn new() -> Self {
    Self {
      expire: ExpireMap::new(),
    }
  }

  pub fn insert(&self, key: K, caller: C, retry: u8) {
    self
      .expire
      .insert(key, Retry { n: retry, caller }, C::ttl());
  }
}

impl<K: Key, C: Caller<K> + Debug> Deref for RetryMap<K, C> {
  type Target = ExpireMap<K, Retry<C>>;
  fn deref(&self) -> &<Self as Deref>::Target {
    &self.expire
  }
}
```


### 关于

本项目隶属于 **人民网络 ([rmw.link](//rmw.link))** 代码计划。

![人民网络海报](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)
