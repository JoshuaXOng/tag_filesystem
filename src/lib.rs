#![cfg_attr(test, allow(unused))]

capnp::generated_code!(pub mod filesystem_capnp);
capnp::generated_code!(pub mod root_capnp);
capnp::generated_code!(pub mod file_capnp);
capnp::generated_code!(pub mod tag_capnp);

drums::define_with_backtrace!();

pub mod cli;
pub mod entries;
pub mod errors;
pub mod files;
pub mod filesystem;
pub mod fuse;
#[cfg(test)]
mod tests;
pub mod inodes;
pub mod journal;
pub mod namespaces;
pub mod os;
pub mod path;
pub mod persistence;
pub mod snapshots;
pub mod storage;
pub mod tags;
pub mod tracing;
pub mod ttl;
pub mod wrappers;
