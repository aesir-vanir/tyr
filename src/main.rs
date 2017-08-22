// Copyright (c) 2017 tyr developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tyr` 0.1.0
#![feature(custom_attribute)]
#![cfg_attr(feature = "cargo-clippy", allow(use_self))]
#![deny(missing_docs)]
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate getset;
#[macro_use]
mod macros;
#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate clap;
extern crate inflector;
extern crate mimir;
extern crate mustache;
extern crate rustc_serialize;
extern crate serde;
extern crate term;

mod context;
mod error;
mod run;
mod tmpl;
mod util;

use std::io::{self, Write};
use std::process;

/// CLI Entry Point
fn main() {
    match run::run() {
        Ok(i) => process::exit(i),
        Err(e) => {
            writeln!(io::stderr(), "{}", e).expect("Unable to write to stderr!");
            process::exit(1)
        }
    }
}
