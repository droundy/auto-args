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

#[test]
fn enum_error_message() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        First { first: String, second: String },
        Second { second: i32 },
        Third { third: u16 },
    }
    println!("help: {}", EnumOpt::help());
    assert!(EnumOpt::help().contains("--first-first "));
    assert!(EnumOpt::help().contains("--first-second "));
    assert!(EnumOpt::help().contains("--second-second "));
    assert!(EnumOpt::help().contains("--third-third "));

    assert_eq!(
        EnumOpt::First {
            first: "hello".to_string(),
            second: "world".to_string()
        },
        EnumOpt::from_iter(&["", "--first-first", "hello", "--first-second", "world"]).unwrap()
    );

    assert_eq!(
        EnumOpt::Second { second: 5 },
        EnumOpt::from_iter(&["", "--second-second", "5"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());

    assert!(EnumOpt::from_iter(&["", "--first-first", "hello", "--second-second", "5"]).is_err());
    assert_eq!(
        EnumOpt::from_iter(&["", "--first-first", "hello"]),
        Err(auto_args::Error::MissingOption(
            "--first-second".to_string()
        ))
    );
    // FIXME the error message depends on the order of arguments in the types in an annoying way.
    // It would be better to check how many flags *could* be found in each variant, rather than
    // quitting after the first failure.

    // assert_eq!(EnumOpt::from_iter(&["", "--first-second", "hello"]),
    //      Err(auto_args::Error::MissingOption("--first-first".to_string())));
}
