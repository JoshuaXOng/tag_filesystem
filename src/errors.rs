use std::error::Error;

pub type Result_<T> = Result<T, Box<dyn Error>>;
