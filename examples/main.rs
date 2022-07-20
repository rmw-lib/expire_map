use anyhow::Result;
use expire_retry_map::add;

fn main() -> Result<()> {
  dbg!(add(1, 2));
  Ok(())
}
