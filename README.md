<!-- EDIT /Users/z/rmw/expire_map/README.md -->

<h1 align="center"> expire_map</h1>
<p align="center">
<a href="#english-readme">English</a>
|
<a href="#中文说明 "> 中文说明 </a>
</p>

---

## English Readme

<!-- EDIT /Users/z/rmw/expire_map/doc/en/readme.md -->

### Use

[→ examples/main.rs](examples/main.rs)

```rust
#![feature(drain_filter)]

use anyhow::Result;

fn main() -> Result<()> {
  let mut v = vec![0, 1, 2];

  v.drain_filter(|x| *x % 2 == 0);

  dbg!(v);
  Ok(())
}
```


### Link

* [benchmark report log](https://rmw-lib.github.io/expire_map/dev/bench/)

### About

This project is part of **[rmw.link](//rmw.link)** Code Project

![rmw.link logo](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)

---

## 中文说明

<!-- EDIT /Users/z/rmw/expire_map/doc/zh/readme.md -->

expire_map : 最大支持 256 个周期超时的无锁字典。

用于网络请求超时和重试。

### 使用

[→ examples/main.rs](examples/main.rs)

```rust
#![feature(drain_filter)]

use anyhow::Result;

fn main() -> Result<()> {
  let mut v = vec![0, 1, 2];

  v.drain_filter(|x| *x % 2 == 0);

  dbg!(v);
  Ok(())
}
```


### 相关连接

* [性能评测日志](https://rmw-lib.github.io/expire_map/dev/bench/)

### 关于

本项目隶属于 **人民网络 ([rmw.link](//rmw.link))** 代码计划。

![人民网络海报](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)
