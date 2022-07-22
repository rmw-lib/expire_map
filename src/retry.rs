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
