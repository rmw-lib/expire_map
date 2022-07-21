#![feature(drain_filter)]

use anyhow::Result;

fn main() -> Result<()> {
  let mut v = vec![0, 1, 2];

  v.drain_filter(|x| *x % 2 == 0);

  dbg!(v);
  Ok(())
}
