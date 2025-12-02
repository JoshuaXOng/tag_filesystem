#![cfg_attr(test, allow(unused))]

capnp::generated_code!(pub mod inodes_capnp);

pub mod cli;
pub mod entries;
pub mod errors;
pub mod files;
pub mod filesystem;
pub mod fuse_;
#[cfg(test)]
mod tests;
pub mod inodes;
pub mod journal;
pub mod namespaces;
pub mod path_;
pub mod serde;
pub mod snapshots;
pub mod storage;
pub mod tags;
pub mod tracing_;
pub mod ttl;
pub mod wrappers;
