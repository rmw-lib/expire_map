<!-- EDIT /Users/z/rmw/expire_map/doc/README.md -->

[English](nglish-readme) | [中文说明](#中文说明)

---

#English Readme

<!-- EDIT /Users/z/rmw/expire_map/doc/en/readme.md -->

## Use

`expire_map` : High concurrency timeout dictionary supporting a maximum of 256 cycles timeout (internally implemented using dashmap).

Unlike the existing rust expire map, there is context object in the parameters of the timeout callback, which avoids wasting memory space for context pointers in each timeout object.

Also, I implement RetryMap based on ExpireMap and can be used for network request timeouts and retries.

## RetryMap usage demo

[→ examples/main.rs](../examples/main.rs)

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

struct Db {}
impl Db {
  pub fn insert(&self, addr: SocketAddrV4, msg: impl AsRef<[u8]>) {
    println!("fail {} {:?}", addr, msg.as_ref());
  }
}

struct Ctx {
  udp: UdpSocket,
  db: Db,
}

impl Ctx {
  fn new() -> Result<Self> {
    let udp = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    Ok(Self { udp, db: Db {} })
  }
}

impl Caller<Ctx, Task> for Msg {
  fn ttl() -> u8 {
    2 // expire after 2 seconds
  }

  fn call(&mut self, ctx: &Ctx, task: &Task) -> u8 {
    let cmd = format!("{} {}#{} {:?}", "call", task.addr, task.id, &self.msg);
    if let Err(err) = ctx.udp.send_to(
      &[&task.id.to_le_bytes()[..], &self.msg[..]].concat(),
      task.addr,
    ) {
      dbg!(err);
    }
    dbg!(cmd);
    Self::ttl()
  }

  fn fail(&mut self, ctx: &Ctx, task: &Task) {
    ctx.db.insert(task.addr, &self.msg)
  }
}

fn main() -> Result<()> {
  let ctx = Ctx::new()?;
  let retry_map = RetryMap::new(ctx);

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

[→ out.txt](../out.txt)

```txt
+ ./sh/run.sh --example main
+ exec cargo run --example main
   Compiling expire_map v0.0.26 (/Users/z/rmw/expire_map)
    Finished dev [unoptimized + debuginfo] target(s) in 1.77s
     Running `/Users/z/rmw/expire_map/target/debug/examples/main`
[examples/main.rs:53] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
1 true
2 true
3 true
4 true
5 true
[examples/main.rs:53] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
6 true
7 true
[examples/main.rs:53] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
8 true
9 true
fail 223.5.5.5:53 [1, 2, 3]
10 false
11 false
12 false
13 false
14 false
15 false
```


## ExpireMap usage demo

The use of ExpireMap can be seen in the RetryMap implementation

[→ src/retry.rs](../src/retry.rs)

```rust
use std::{default::Default, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

pub trait Caller<Ctx, K> {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&mut self, ctx: &Ctx, key: &K) -> u8;
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
      self.caller.call(ctx, key)
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
  pub fn remove(&self, key: K) -> Option<C> {
    if let Some(r) = self.expire.remove(key) {
      Some(r.caller)
    } else {
      None
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


---

#中文说明

<!-- EDIT /Users/z/rmw/expire_map/doc/zh/readme.md -->

`expire_map` : 最大支持 256 个周期超时的高并发字典（内部使用 dashmap 实现）。

和现有的 rust 超时字典不同，在超时回调的参数中有上下文对象，这样可以避免在每个超时对象中浪费内存空间放置上下文指针。

同时，我还基于 ExpireMap 实现了 RetryMap，可以用于网络请求超时和重试。

## RetryMap 使用演示

[→ examples/main.rs](../examples/main.rs)

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

struct Db {}
impl Db {
  pub fn insert(&self, addr: SocketAddrV4, msg: impl AsRef<[u8]>) {
    println!("fail {} {:?}", addr, msg.as_ref());
  }
}

struct Ctx {
  udp: UdpSocket,
  db: Db,
}

impl Ctx {
  fn new() -> Result<Self> {
    let udp = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    Ok(Self { udp, db: Db {} })
  }
}

impl Caller<Ctx, Task> for Msg {
  fn ttl() -> u8 {
    2 // expire after 2 seconds
  }

  fn call(&mut self, ctx: &Ctx, task: &Task) -> u8 {
    let cmd = format!("{} {}#{} {:?}", "call", task.addr, task.id, &self.msg);
    if let Err(err) = ctx.udp.send_to(
      &[&task.id.to_le_bytes()[..], &self.msg[..]].concat(),
      task.addr,
    ) {
      dbg!(err);
    }
    dbg!(cmd);
    Self::ttl()
  }

  fn fail(&mut self, ctx: &Ctx, task: &Task) {
    ctx.db.insert(task.addr, &self.msg)
  }
}

fn main() -> Result<()> {
  let ctx = Ctx::new()?;
  let retry_map = RetryMap::new(ctx);

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

[→ out.txt](../out.txt)

```txt
+ ./sh/run.sh --example main
+ exec cargo run --example main
   Compiling expire_map v0.0.26 (/Users/z/rmw/expire_map)
    Finished dev [unoptimized + debuginfo] target(s) in 1.77s
     Running `/Users/z/rmw/expire_map/target/debug/examples/main`
[examples/main.rs:53] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
1 true
2 true
3 true
4 true
5 true
[examples/main.rs:53] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
6 true
7 true
[examples/main.rs:53] cmd = "call 223.5.5.5:53#12345 [1, 2, 3]"
8 true
9 true
fail 223.5.5.5:53 [1, 2, 3]
10 false
11 false
12 false
13 false
14 false
15 false
```


## ExpireMap 使用演示

ExpireMap 的使用可以参见 RetryMap 的实现

[→ src/retry.rs](../src/retry.rs)

```rust
use std::{default::Default, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

pub trait Caller<Ctx, K> {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&mut self, ctx: &Ctx, key: &K) -> u8;
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
      self.caller.call(ctx, key)
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
  pub fn remove(&self, key: K) -> Option<C> {
    if let Some(r) = self.expire.remove(key) {
      Some(r.caller)
    } else {
      None
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

