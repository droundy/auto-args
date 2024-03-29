// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[test]
fn tuple_struct() {
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Opt(i32);
    println!("help: {}", Opt::help());
    assert!(!Opt::help().contains("--first"));

    assert_eq!(Opt(7), Opt::from_iter(&["", "7"]).unwrap());

    assert!(Opt::from_iter(&[""]).is_err());

    assert!(Opt::from_iter(&["hello"]).is_err());
}

#[test]
fn unit_struct() {
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Opt;
    println!("help: {}", Opt::help());
    assert!(!Opt::help().contains("--first"));

    assert_eq!(Opt, Opt::from_iter(&[""]).unwrap());
}
