use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum SodiumOxideError {
    InitPull,
    InitPush,
    Pull,
    Push,
    InvalidKeyLength,
}

impl Display for SodiumOxideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SodiumOxideError {}
