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
    First {
        name: Option<String>,
    },
    Second {
        values: Vec<usize>,
    },
}


#[test]
fn craziness() {
    println!("help: {}", Params::help());
    assert!(!Option::<String>::REQUIRES_INPUT);
    assert!(Params::help().contains("--first-name "));
    assert!(Params::help().contains("--first "));
    assert_eq!(Ok(Params::First { name: None }), Params::from_iter(&["", "--first"]));
    assert_eq!(Ok(Params::Second { values: Vec::new() }),
               Params::from_iter(&["", "--second"]));
    assert_eq!(Ok(Params::Second { values: vec![1] }),
               Params::from_iter(&["", "--second", "--second-values", "1"]));
    assert!(Params::from_iter(&["", "--second-values"]).is_err());
}
