use std::{fmt::Debug, ops::Deref};

use crate::{expire_map::Key, ExpireMap, OnExpire};

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
  fn fail(&self);
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
      self.caller.fail();
      0
    } else {
      self.n = n;
      C::ttl()
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct RetryMap<K: Key, C: Caller + Debug> {
  pub expire: ExpireMap<K, Retry<C>>,
}

impl<K: Key, C: Caller + Debug> RetryMap<K, C> {
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

impl<K: Key, C: Caller + Debug> Deref for RetryMap<K, C> {
  type Target = ExpireMap<K, Retry<C>>;
  fn deref(&self) -> &<Self as Deref>::Target {
    &self.expire
  }
}
