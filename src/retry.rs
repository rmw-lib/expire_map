use crate::OnExpire;

/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Call {
  fn call(&self);
}

#[derive(Debug, Default)]
pub struct Retry<C: Call> {
  n: u8,
  caller: C,
}

impl<C: Call> OnExpire for Retry<C> {
  fn on_expire(&mut self) -> u8 {
    self.caller.call();
    let n = self.n.wrapping_sub(1);
    self.n = n;
    n
  }
}
