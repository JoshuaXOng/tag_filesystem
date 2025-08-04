#![cfg_attr(test, allow(unused))]

use std::error::Error;

mod entries;
mod errors;
mod files;
mod filesystem;
mod fuse_;
#[cfg(test)]
mod tests;
mod inodes;
mod queries;
mod rollback;
mod tags;
mod ttl;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
