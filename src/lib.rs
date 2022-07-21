#![feature(trait_alias)]
#![feature(drain_filter)]

mod expire_map;
pub use crate::expire_map::{ExpireMap, OnExpire};

#[cfg(feature = "retry")]
mod retry;

#[cfg(feature = "retry")]
pub use retry::{Caller, RetryMap};
