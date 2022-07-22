use std::{default::Default, fmt::Debug, ops::Deref};

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
  fn call(&mut self, key: &K);
  fn fail(&mut self, key: &K);
}

#[derive(Debug, Default)]
pub struct Retry<C> {
  n: u8,
  caller: C,
}

impl<K, C: Caller<K>> OnExpire<K> for Retry<C> {
  fn on_expire(&mut self, key: &K) -> u8 {
    let n = self.n.wrapping_sub(1);
    if n == 0 {
      self.caller.fail(key);
      0
    } else {
      self.n = n;
      self.caller.call(key);
      C::ttl()
    }
  }
}

pub trait Task<K> = Caller<K> + Debug;

#[derive(Debug, Default)]
pub struct RetryMap<K: Key, C: Task<K>> {
  pub expire: ExpireMap<K, Retry<C>>,
}

impl<K: Key, C: Task<K>> Clone for RetryMap<K, C> {
  fn clone(&self) -> Self {
    Self {
      expire: self.expire.clone(),
    }
  }
}

impl<K: Key, C: Task<K>> RetryMap<K, C> {
  pub fn new() -> Self {
    Self {
      expire: ExpireMap::new(),
    }
  }

  pub fn insert(&self, key: K, mut caller: C, retry: u8) {
    caller.call(&key);
    self
      .expire
      .insert(key, Retry { n: retry, caller }, C::ttl());
  }
}

impl<K: Key, C: Task<K>> Deref for RetryMap<K, C> {
  type Target = ExpireMap<K, Retry<C>>;
  fn deref(&self) -> &<Self as Deref>::Target {
    &self.expire
  }
}
