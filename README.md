<!-- EDIT /Users/z/rmw/expire_map/README.md -->

# expire_map

<a href="https://docs.rs/expire_map"><img src="https://img.shields.io/badge/RUST-API%20DOC-blue?style=for-the-badge&logo=docs.rs&labelColor=333" alt="Api Doc"></a>

[English](#english-readme) | [中文说明](#中文说明)

---

## English Readme

<!-- EDIT /Users/z/rmw/expire_map/doc/en/readme.md -->

## Use

`expire_map` : High concurrency dictionary supporting a maximum of 256 cycles timeout (internally implemented using dashmap).

Also, I implement RetryMap based on ExpireMap and can be used for network request timeouts and retries.

## RetryMap usage demo

[→ examples/main.rs](examples/main.rs)

```rust
use std::{
  net::{Ipv4Addr, SocketAddrV4, UdpSocket},
  time::Duration,
};

use anyhow::Result;
use async_std::task::{block_on, sleep, spawn};
use expire_map::{Caller, RetryMap};

#[derive(Debug)]
struct Msg {
  msg: Box<[u8]>,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
struct Task {
  addr: SocketAddrV4,
  id: u16,
}

impl Caller<UdpSocket, Task> for Msg {
  fn ttl() -> u8 {
    2 // 2 seconds timeout
  }

  fn call(&mut self, udp: &UdpSocket, task: &Task) {
    let cmd = format!("{} {}#{} {:?}", "call", task.addr, task.id, &self.msg);
    if let Err(err) = udp.send_to(
      &[&task.id.to_le_bytes()[..], &self.msg[..]].concat(),
      task.addr,
    ) {
      dbg!(err);
    }
    dbg!(cmd);
  }

  fn fail(&mut self, _: &UdpSocket, task: &Task) {
    let cmd = format!("{} {}#{} {:?}", "fail", task.addr, task.id, &self.msg);
    dbg!(cmd);
  }
}

fn main() -> Result<()> {
  let udp = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;

  let retry_map = RetryMap::new(udp);

  let msg = Msg {
    msg: Box::from(&[1, 2, 3][..]),
  };

  let task = Task {
    id: 12345,
    addr: "223.5.5.5:53".parse()?,
  };

  let retry_times = 3; // 重试次数是3次

  let expireer = retry_map.clone();

  let handle = spawn(async move {
    let mut do_expire = 0;
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      do_expire += 1;
      let exist = expireer.get(&task).is_some();
      println!("{} {}", do_expire, exist);
    }
  });

  // will run call() when insert
  retry_map.insert(task, msg, retry_times);

  retry_map.renew(task, 5);
  //dbg!(retry_map.get(&task).unwrap().value());
  //dbg!(retry_map.get_mut(&task).unwrap().key());
  //retry_map.remove(task);

  block_on(handle);
  Ok(())
}
```


Output

[→ out.txt](out.txt)

```txt
+ ./sh/run.sh --example main
+ exec cargo run --example main
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `/Users/z/rmw/expire_map/target/debug/examples/main`
[examples/main.rs:24] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
1 true
2 true
3 true
4 true
5 true
[examples/main.rs:24] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
6 true
7 true
[examples/main.rs:24] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
8 true
9 true
[examples/main.rs:29] cmd = "fail 223.5.5.5:53#12345 [1, 2, 3]"
10 false
11 false
12 false
13 false
```


## ExpireMap usage demo

The use of ExpireMap can be seen in the RetryMap implementation

[→ src/retry.rs](src/retry.rs)

```rust
use std::{default::Default, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Caller<Ctx, K> {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&mut self, ctx: &Ctx, key: &K);
  fn fail(&mut self, ctx: &Ctx, key: &K);
}

#[derive(Default)]
pub struct Retry<C> {
  n: u8,
  caller: C,
}

impl<Ctx, K, C: Caller<Ctx, K>> OnExpire<Ctx, K> for Retry<C> {
  fn on_expire(&mut self, ctx: &Ctx, key: &K) -> u8 {
    let n = self.n.wrapping_sub(1);
    if n == 0 {
      self.caller.fail(ctx, key);
      0
    } else {
      self.n = n;
      self.caller.call(ctx, key);
      C::ttl()
    }
  }
}

pub trait Task<Ctx, K> = Caller<Ctx, K>;

pub struct RetryMap<Ctx, K: Key, C: Task<Ctx, K>> {
  pub expire: ExpireMap<Ctx, K, Retry<C>>,
}

impl<Ctx, K: Key, C: Task<Ctx, K>> Clone for RetryMap<Ctx, K, C> {
  fn clone(&self) -> Self {
    Self {
      expire: self.expire.clone(),
    }
  }
}

impl<Ctx, K: Key, C: Task<Ctx, K>> RetryMap<Ctx, K, C> {
  pub fn new(ctx: Ctx) -> Self {
    Self {
      expire: ExpireMap::new(ctx),
    }
  }

  pub fn insert(&self, key: K, mut caller: C, retry: u8) {
    caller.call(&self.ctx, &key);
    self
      .expire
      .insert(key, Retry { n: retry, caller }, C::ttl());
  }
}

impl<Ctx, K: Key, C: Task<Ctx, K>> Deref for RetryMap<Ctx, K, C> {
  type Target = ExpireMap<Ctx, K, Retry<C>>;
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

## RetryMap 使用演示

[→ examples/main.rs](examples/main.rs)

```rust
use std::{
  net::{Ipv4Addr, SocketAddrV4, UdpSocket},
  time::Duration,
};

use anyhow::Result;
use async_std::task::{block_on, sleep, spawn};
use expire_map::{Caller, RetryMap};

#[derive(Debug)]
struct Msg {
  msg: Box<[u8]>,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
struct Task {
  addr: SocketAddrV4,
  id: u16,
}

impl Caller<UdpSocket, Task> for Msg {
  fn ttl() -> u8 {
    2 // 2 seconds timeout
  }

  fn call(&mut self, udp: &UdpSocket, task: &Task) {
    let cmd = format!("{} {}#{} {:?}", "call", task.addr, task.id, &self.msg);
    if let Err(err) = udp.send_to(
      &[&task.id.to_le_bytes()[..], &self.msg[..]].concat(),
      task.addr,
    ) {
      dbg!(err);
    }
    dbg!(cmd);
  }

  fn fail(&mut self, _: &UdpSocket, task: &Task) {
    let cmd = format!("{} {}#{} {:?}", "fail", task.addr, task.id, &self.msg);
    dbg!(cmd);
  }
}

fn main() -> Result<()> {
  let udp = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;

  let retry_map = RetryMap::new(udp);

  let msg = Msg {
    msg: Box::from(&[1, 2, 3][..]),
  };

  let task = Task {
    id: 12345,
    addr: "223.5.5.5:53".parse()?,
  };

  let retry_times = 3; // 重试次数是3次

  let expireer = retry_map.clone();

  let handle = spawn(async move {
    let mut do_expire = 0;
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      do_expire += 1;
      let exist = expireer.get(&task).is_some();
      println!("{} {}", do_expire, exist);
    }
  });

  // will run call() when insert
  retry_map.insert(task, msg, retry_times);

  retry_map.renew(task, 5);
  //dbg!(retry_map.get(&task).unwrap().value());
  //dbg!(retry_map.get_mut(&task).unwrap().key());
  //retry_map.remove(task);

  block_on(handle);
  Ok(())
}
```


运行输出

[→ out.txt](out.txt)

```txt
+ ./sh/run.sh --example main
+ exec cargo run --example main
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `/Users/z/rmw/expire_map/target/debug/examples/main`
[examples/main.rs:24] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
1 true
2 true
3 true
4 true
5 true
[examples/main.rs:24] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
6 true
7 true
[examples/main.rs:24] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
8 true
9 true
[examples/main.rs:29] cmd = "fail 223.5.5.5:53#12345 [1, 2, 3]"
10 false
11 false
12 false
13 false
```


## ExpireMap 使用演示

ExpireMap 的使用可以参见 RetryMap 的实现

[→ src/retry.rs](src/retry.rs)

```rust
use std::{default::Default, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Caller<Ctx, K> {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&mut self, ctx: &Ctx, key: &K);
  fn fail(&mut self, ctx: &Ctx, key: &K);
}

#[derive(Default)]
pub struct Retry<C> {
  n: u8,
  caller: C,
}

impl<Ctx, K, C: Caller<Ctx, K>> OnExpire<Ctx, K> for Retry<C> {
  fn on_expire(&mut self, ctx: &Ctx, key: &K) -> u8 {
    let n = self.n.wrapping_sub(1);
    if n == 0 {
      self.caller.fail(ctx, key);
      0
    } else {
      self.n = n;
      self.caller.call(ctx, key);
      C::ttl()
    }
  }
}

pub trait Task<Ctx, K> = Caller<Ctx, K>;

pub struct RetryMap<Ctx, K: Key, C: Task<Ctx, K>> {
  pub expire: ExpireMap<Ctx, K, Retry<C>>,
}

impl<Ctx, K: Key, C: Task<Ctx, K>> Clone for RetryMap<Ctx, K, C> {
  fn clone(&self) -> Self {
    Self {
      expire: self.expire.clone(),
    }
  }
}

impl<Ctx, K: Key, C: Task<Ctx, K>> RetryMap<Ctx, K, C> {
  pub fn new(ctx: Ctx) -> Self {
    Self {
      expire: ExpireMap::new(ctx),
    }
  }

  pub fn insert(&self, key: K, mut caller: C, retry: u8) {
    caller.call(&self.ctx, &key);
    self
      .expire
      .insert(key, Retry { n: retry, caller }, C::ttl());
  }
}

impl<Ctx, K: Key, C: Task<Ctx, K>> Deref for RetryMap<Ctx, K, C> {
  type Target = ExpireMap<Ctx, K, Retry<C>>;
  fn deref(&self) -> &<Self as Deref>::Target {
    &self.expire
  }
}
```


### 关于

本项目隶属于 **人民网络 ([rmw.link](//rmw.link))** 代码计划。

![人民网络海报](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)
