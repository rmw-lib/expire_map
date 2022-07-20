#![feature(trait_alias)]

mod retry;

use std::{
  collections::BTreeMap,
  sync::atomic::{AtomicU8, Ordering::Relaxed},
};

use atomic_traits::fetch::Add;
use num_traits::bounds::UpperBounded;
use parking_lot::RwLock;

#[derive(Debug, Default)]
pub struct ExpireMap<Id, Task, N = u8, AN = AtomicU8> {
  map: RwLock<BTreeMap<N, Id>>,
  task: RwLock<BTreeMap<Id, Task>>,
  n: AN,
}

pub trait Num = UpperBounded + From<u8> + Eq;

impl<Id, Task, N: Num, AN: Add<Type = N>> ExpireMap<Id, Task, N, AN> {
  pub fn compact(&self) {
    let n = self.n.fetch_add(1u8.into(), Relaxed);
    if n == 0.into() {}
  }
}
