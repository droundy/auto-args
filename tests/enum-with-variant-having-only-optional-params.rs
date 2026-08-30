// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[derive(Debug, AutoArgs, PartialEq, Eq)]
enum Params {
    First { name: Option<String> },
    Second { values: Vec<usize> },
}

#[test]
fn craziness() {
    println!("help: {}", Params::help());
    assert!(!Option::<String>::REQUIRES_INPUT);
    assert!(Params::help().contains("--name "));
    assert!(Params::help().contains("first"));

    // Under flag-style dispatch, an all-optional-fields variant needed
    // an extra bare `--first` flag on top of its (optional) fields, just
    // to disambiguate "the First variant was selected with nothing
    // supplied" from "nothing was supplied at all". With subcommand
    // dispatch the keyword itself is that disambiguation, so no such
    // workaround is needed any more.
    assert_eq!(
        Ok(Params::First { name: None }),
        Params::from_iter(&["", "first"])
    );
    assert_eq!(
        Ok(Params::Second { values: Vec::new() }),
        Params::from_iter(&["", "second"])
    );
    assert_eq!(
        Ok(Params::Second { values: vec![1] }),
        Params::from_iter(&["", "second", "--values", "1"])
    );
    assert!(Params::from_iter(&["", "second", "--values"]).is_err());
}
