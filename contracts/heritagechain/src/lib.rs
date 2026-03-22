#![no_std]

pub mod types;
pub mod storage;
pub mod services;
pub mod contract;

pub use crate::contract::*;

#[cfg(test)]
mod test;
