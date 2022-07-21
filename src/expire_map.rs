use std::{
  cmp::Eq,
  fmt::Debug,
  hash::Hash,
  marker::Copy,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicU8, Ordering::Relaxed},
};

use array_macro::array;
use dashmap::{
  mapref::one::{Ref, RefMut},
  DashMap, DashSet,
};

pub trait OnExpire {
  /// expire when return 0 else renew n duration
  fn on_expire(&mut self) -> u8;
}

#[derive(Debug)]
pub struct _Task<Task> {
  expire_on: u8,
  task: Task,
}

impl<Task> DerefMut for _Task<Task> {
  fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
    &mut self.task
  }
}

impl<Task> Deref for _Task<Task> {
  type Target = Task;
  fn deref(&self) -> &Self::Target {
    &self.task
  }
}

pub trait Key = Copy + Hash + Debug + Eq;
pub trait Task = Debug + OnExpire;

#[derive(Debug)]
pub struct ExpireMap<K: Key, T: Task> {
  li: [DashSet<K>; u8::MAX as _],
  task: DashMap<K, _Task<T>>,
  n: AtomicU8,
}

impl<K: Key, T: Task> Default for ExpireMap<K, T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, K: Key, T: Task> ExpireMap<K, T> {
  pub fn new() -> Self {
    Self {
      li: array![_=>DashSet::new();u8::MAX as usize],
      task: DashMap::new(),
      n: AtomicU8::new(0),
    }
  }

  pub fn do_expire(&self) {
    let n = self.n.fetch_add(1, Relaxed) as usize;
    let li = &self.li[n];
    for key in li.iter() {
      li.remove(&key);
      if let Some(mut t) = self.task.get_mut(&key) {
        match t.task.on_expire() {
          0 => {
            self.task.remove(&key);
          }
          n => {
            let n = self.n.load(Relaxed).wrapping_add(n);
            t.expire_on = n;
            self.li[n as usize].insert(*key);
          }
        }
      }
    }
  }

  pub fn get(&'a self, key: &K) -> Option<Ref<'a, K, _Task<T>>> {
    self.task.get(&key)
  }

  pub fn get_mut(&'a self, key: &K) -> Option<RefMut<'a, K, _Task<T>>> {
    self.task.get_mut(&key)
  }

  pub fn remove(&self, key: K) {
    if let Some(t) = self.task.get(&key) {
      self.li[t.expire_on as usize].remove(&key);
      self.task.remove(&key);
    }
  }

  pub fn insert(&self, key: K, task: T, expire: u8) {
    let n = self.n.load(Relaxed).wrapping_add(expire);
    self.task.insert(key, _Task { expire_on: n, task });
    self.li[n as usize].insert(key);
  }
}
