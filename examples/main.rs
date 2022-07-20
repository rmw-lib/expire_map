use anyhow::Result;
use expire_map::add;

fn main() -> Result<()> {
  dbg!(add(1, 2));
  Ok(())
}
