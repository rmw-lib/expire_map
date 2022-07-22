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
