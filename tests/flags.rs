// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[test]
fn unique_flag() {
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Opt {
        /// Documentation for alice
        alice: bool,
    }

    assert!(Opt::help().contains("--alice"));
    assert!(Opt::help().contains("Documentation for alice"));

    assert_eq!(
        Opt { alice: true },
        Opt::from_iter(&["", "--alice"]).unwrap()
    );

    assert_eq!(Opt { alice: false }, Opt::from_iter(&[""]).unwrap());

    assert!(Opt::from_iter(&["", "--bob"]).is_err());
}

#[test]
fn flag_with_underscores() {
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Opt {
        /// Documentation for awesomeness
        this_is_awesome: bool,
    }

    println!("{}", Opt::help());
    assert!(Opt::help().contains("--this-is-awesome"));
    assert!(Opt::help().contains("Documentation for awesomeness"));

    assert_eq!(
        Opt {
            this_is_awesome: true
        },
        Opt::from_iter(&["", "--this-is-awesome"]).unwrap()
    );

    assert_eq!(
        Opt {
            this_is_awesome: false
        },
        Opt::from_iter(&[""]).unwrap()
    );

    assert!(Opt::from_iter(&["", "--bob"]).is_err());
}
