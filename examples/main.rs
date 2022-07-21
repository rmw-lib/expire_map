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
    dbg!(("call", self));
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
    let mut n = 0;
    loop {
      sleep(Duration::from_secs(1)).await;
      expireer.do_expire();
      n += 1;
      dbg!(format!("do expire {}", n));
    }
  });

  retry_map.insert(task_id, task, retry_times);
  dbg!(retry_map.get(&task_id).unwrap().value());

  block_on(handle);
  Ok(())
}
