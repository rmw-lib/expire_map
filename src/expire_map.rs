use std::{
  cmp::Eq,
  fmt::Debug,
  hash::Hash,
  marker::Copy,
  ops::{Deref, DerefMut},
  sync::{
    atomic::{AtomicU8, Ordering::Relaxed},
    Arc,
  },
};

use array_macro::array;
use dashmap::{
  mapref::one::{Ref, RefMut},
  DashMap, DashSet,
};

pub trait OnExpire<K> {
  /// expire when return 0 else renew n duration
  fn on_expire(&mut self, key: &K) -> u8;
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
pub trait Task<K> = Debug + OnExpire<K>;

const SIZE: usize = u8::MAX as usize + 1;

#[derive(Debug)]
pub struct Inner<K: Key, T: Task<K>> {
  li: [DashSet<K>; SIZE],
  task: DashMap<K, _Task<T>>,
  n: AtomicU8,
}

#[derive(Debug, Default)]
pub struct ExpireMap<K: Key, T: Task<K>> {
  inner: Arc<Inner<K, T>>,
}

impl<K: Key, T: Task<K>> Clone for ExpireMap<K, T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<K: Key, T: Task<K>> ExpireMap<K, T> {
  pub fn new() -> Self {
    Self {
      inner: Arc::new(Inner::new()),
    }
  }
}

impl<K: Key, T: Task<K>> Deref for ExpireMap<K, T> {
  type Target = Inner<K, T>;
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<K: Key, T: Task<K>> Default for Inner<K, T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, K: Key, T: Task<K>> Inner<K, T> {
  pub fn new() -> Self {
    Self {
      li: array![_=>DashSet::new();SIZE],
      task: DashMap::new(),
      n: AtomicU8::new(0),
    }
  }

  pub fn do_expire(&self) {
    let n = self.n.fetch_add(1, Relaxed);
    let li = &self.li[n as usize];
    for key in li.iter() {
      if if let Some(mut t) = self.task.get_mut(&key) {
        match t.task.on_expire(&key) {
          0 => true,
          x => {
            t.expire_on = n.wrapping_add(x);
            self.li[t.expire_on as usize].insert(*key);
            false
          }
        }
      } else {
        false
      } {
        self.task.remove(&key);
      }
    }
    // 因 dashmap 没有 drain_filter (https://github.com/xacrimon/dashmap/issues/224) ，可能会导致一些内存泄露（iter和clear之间插入了新条目），但是当expire大于1，且间隔为秒的时候，基本不可能泄露，因为新插入的条目都是在n+expire > n+1，对于我的场景，足够了
    li.clear();
  }

  pub fn get(&'a self, key: &K) -> Option<Ref<'a, K, _Task<T>>> {
    self.task.get(key)
  }

  pub fn get_mut(&'a self, key: &K) -> Option<RefMut<'a, K, _Task<T>>> {
    self.task.get_mut(key)
  }

  pub fn remove(&self, key: K) {
    if let Some(t) = self.task.get(&key) {
      self.li[t.expire_on as usize].remove(&key);
      self.task.remove(&key);
    }
  }

  pub fn renew(&'a self, key: K, expire: u8) -> Option<RefMut<'a, K, _Task<T>>> {
    let mut r = self.task.get_mut(&key);
    if let Some(ref mut r) = r {
      let n = self.n.load(Relaxed).wrapping_add(expire);
      if n != r.expire_on {
        self.li[r.expire_on as usize].remove(&key);
        self.li[n as usize].insert(key);
        r.expire_on = n;
      }
    }
    r
  }

  pub fn insert(&self, key: K, task: T, expire: u8) {
    let n = self.n.load(Relaxed).wrapping_add(expire);
    self.task.insert(key, _Task { expire_on: n, task });
    self.li[n as usize].insert(key);
  }
}
