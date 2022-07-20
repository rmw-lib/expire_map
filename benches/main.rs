use anyhow::Error;
use criterion::{criterion_group, criterion_main, Criterion};
use expire_retry_map::add;
use rand::Rng;

fn criterion_benchmark(c: &mut Criterion) {
  let mut rng = rand::thread_rng();

  macro_rules! run {
    ($name:ident $func:expr) => {
      c.bench_function(stringify!($name), $func);
    };
  }

  run!(
    add | b | {
      b.iter(|| {
        add(rng.gen(), rng.gen());
        Ok::<_, Error>(())
      })
    }
  );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
