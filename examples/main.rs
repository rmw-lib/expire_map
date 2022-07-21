use std::net::{SocketAddr, SocketAddrV4};

use anyhow::Result;
use async_std::task::spawn;
use expire_map::{Caller, ExpireMap};

struct Task {
  addr: SocketAddrV4,
  msg: Box<u8>,
}

fn main() -> Result<()> {
  //let expire_map = ExpireMap::new();
  let task_id = 1;
  //expire_map.insert();

  Ok(())
}
