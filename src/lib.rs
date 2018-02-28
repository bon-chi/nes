#![recursion_limit = "1024"]
#![feature(untagged_unions)]
#[macro_use]
extern crate error_chain;

mod errors {
    error_chain!{}
}

pub mod nes;
