// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[test]
fn enum_error_message() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        First { first: String, second: String },
        Second { second: i32 },
        Third { third: u16 },
    }
    println!("help: {}", EnumOpt::help());
    assert!(EnumOpt::help().contains("--first "));
    assert!(EnumOpt::help().contains("--second "));
    assert!(EnumOpt::help().contains("--third "));

    assert_eq!(
        EnumOpt::First {
            first: "hello".to_string(),
            second: "world".to_string()
        },
        EnumOpt::from_iter(&["", "first", "--first", "hello", "--second", "world"]).unwrap()
    );

    assert_eq!(
        EnumOpt::Second { second: 5 },
        EnumOpt::from_iter(&["", "second", "--second", "5"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());

    assert_eq!(
        EnumOpt::from_iter(&["", "first", "--first", "hello"]),
        Err(auto_args::Error::MissingOption("--second".to_string()))
    );

    // Because keyword dispatch is now deterministic (the "first" keyword
    // unambiguously selects the `First` variant before any of its own
    // fields are parsed), the error precisely reflects which field of
    // *that* variant is missing, regardless of field order. This used to
    // be unreliable (see the old FIXME here) when every variant was
    // tried in turn against the same argument pool.
    assert_eq!(
        EnumOpt::from_iter(&["", "first", "--second", "hello"]),
        Err(auto_args::Error::MissingOption("--first".to_string()))
    );
}
