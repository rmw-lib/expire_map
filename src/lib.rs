#![feature(trait_alias)]

use std::{
  cmp::Ord,
  collections::BTreeMap,
  fmt::Debug,
  marker::Copy,
  sync::atomic::{AtomicU8, Ordering::Relaxed},
};

use parking_lot::RwLock;

mod retry;

pub trait OnExpire {
  fn on_expire(&mut self) -> u8;
}

pub trait Key = Copy + Ord + Debug;
pub trait Task = Debug + OnExpire;

pub struct ExpireMap<K: Key, T: Task> {
  pub li: [RwLock<Vec<K>>; u8::MAX as _],
  pub task: RwLock<BTreeMap<K, T>>,
  pub n: AtomicU8,
}

impl<K: Key, T: Task> ExpireMap<K, T> {
  pub fn do_expire(&self) {
    let n = self.n.fetch_add(1, Relaxed) as usize;
    let li = self.li[n].read();
    *self.li[0].write() = vec![];
    for i in li.iter() {
      if let Some(task) = self.task.write().get_mut(i) {
        match task.on_expire() {
          n => {}
          0 => {
            dbg!(&task);
          }
        }
      }
    }
  }

  pub fn add(&self, key: K, task: T, expire: u8) {
    let mut li = self.li[self.n.load(Relaxed).wrapping_add(expire) as usize].write();
    li.push(key);
    self.task.write().insert(key, task);
  }
}
