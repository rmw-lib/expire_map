/*
   btree
   超时时间 id
   重试函数
   重试次数
   失败


*/

pub trait Task {
  fn run(&self) -> bool;
}

pub struct Retry<T: Task, N = u8> {
  n: N,
  task: T,
}
