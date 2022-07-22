use std::{
  cmp::Eq,
  default::Default,
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

pub trait OnExpire<Ctx, K> {
  /// expire when return 0 else renew n duration
  fn on_expire(&mut self, ctx: &Ctx, key: &K) -> u8;
}

#[derive(Debug)]
pub struct ExpireOn<Task> {
  expire_on: u8,
  task: Task,
}

impl<Task> DerefMut for ExpireOn<Task> {
  fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
    &mut self.task
  }
}

impl<Task> Deref for ExpireOn<Task> {
  type Target = Task;
  fn deref(&self) -> &Self::Target {
    &self.task
  }
}

pub trait Key = Copy + Hash + Eq;
pub trait Task<Ctx, K> = OnExpire<Ctx, K>;

const SIZE: usize = u8::MAX as usize + 1;

pub struct Inner<Ctx, K: Key, T: Task<Ctx, K>> {
  li: [DashSet<K>; SIZE],
  task: DashMap<K, ExpireOn<T>>,
  n: AtomicU8,
  pub ctx: Ctx,
}

pub struct ExpireMap<Ctx, K: Key, T: Task<Ctx, K>> {
  inner: Arc<Inner<Ctx, K, T>>,
}

impl<Ctx, K: Key, T: Task<Ctx, K>> Clone for ExpireMap<Ctx, K, T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<Ctx, K: Key, T: Task<Ctx, K>> ExpireMap<Ctx, K, T> {
  pub fn new(ctx: Ctx) -> Self {
    Self {
      inner: Arc::new(Inner::new(ctx)),
    }
  }
}

impl<Ctx, K: Key, T: Task<Ctx, K>> Deref for ExpireMap<Ctx, K, T> {
  type Target = Inner<Ctx, K, T>;
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<'a, Ctx, K: Key, T: Task<Ctx, K>> Inner<Ctx, K, T> {
  pub fn new(ctx: Ctx) -> Self {
    Self {
      li: array![_=>DashSet::new();SIZE],
      task: DashMap::new(),
      n: AtomicU8::new(0),
      ctx,
    }
  }

  pub fn do_expire(&self) {
    let n = self.n.fetch_add(1, Relaxed);
    let li = &self.li[n as usize];
    for key in li.iter() {
      if if let Some(mut t) = self.task.get_mut(&key) {
        match t.task.on_expire(&self.ctx, &key) {
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

  pub fn renew_or_create(
    &'a self,
    key: K,
    create: impl Fn() -> T,
    expire: u8,
  ) -> RefMut<'a, K, ExpireOn<T>> {
    loop {
      if let Some(r) = self.renew(key, expire) {
        return r;
      }
      self.insert(key, create(), expire);
    }
  }

  pub fn remove(&self, key: K) {
    if let Some(t) = self.task.get(&key) {
      self.li[t.expire_on as usize].remove(&key);
      self.task.remove(&key);
    }
  }

  pub fn renew(&'a self, key: K, expire: u8) -> Option<RefMut<'a, K, ExpireOn<T>>> {
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
    self.task.insert(key, ExpireOn { expire_on: n, task });
    self.li[n as usize].insert(key);
  }
}

macro_rules! can_mut {
  ($ref:ident,$get:ident) => {
    impl<'a, Ctx, K: Key, T: Task<Ctx, K>> Inner<Ctx, K, T> {
      pub fn $get(&'a self, key: &K) -> Option<$ref<'a, K, ExpireOn<T>>> {
        self.task.$get(key)
      }
    }
  };
}

can_mut!(Ref, get);
can_mut!(RefMut, get_mut);
