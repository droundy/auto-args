// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate auto_args;

use auto_args::AutoArgs;

#[derive(Debug, AutoArgs)]
enum Params {
    ResumeFrom(String),
    _Params {
        sys: SquareWellParams,
    },
}

#[derive(Debug, AutoArgs)]
struct SquareWellParams {
    well_width: String,
}

#[test]
fn craziness() {
    println!("help: {}", Params::help());
    assert!(Params::help().contains("--resume-from "));
}
