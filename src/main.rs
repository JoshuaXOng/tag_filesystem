use std::error::Error;

mod errors;
mod filesystem;
mod fuse_;
#[cfg(test)]
mod tests;
mod ttl;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
