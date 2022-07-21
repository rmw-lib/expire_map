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

#[derive(Debug, Default, Clone)]
pub struct RetryMap<K: Key, C: Caller<K> + Debug> {
  pub expire: ExpireMap<K, Retry<C>>,
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
