use crate::OnExpire;

/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Caller {
  /// Time-To-Live
  fn ttl() -> u8;
  fn call(&self);
}

#[derive(Debug, Default)]
pub struct Retry<C: Caller> {
  n: u8,
  caller: C,
}

impl<C: Caller> OnExpire for Retry<C> {
  fn on_expire(&mut self) -> u8 {
    self.caller.call();
    let n = self.n.wrapping_sub(1);
    if n == 0 {
      0
    } else {
      self.n = n;
      C::ttl()
    }
  }
}
