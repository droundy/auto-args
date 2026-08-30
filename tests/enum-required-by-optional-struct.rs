// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[derive(Debug, AutoArgs, PartialEq)]
enum Params {
    ResumeFrom(String),
    _Params { sys: SquareWellParams },
}

#[derive(Debug, AutoArgs, PartialEq)]
enum Dimensions {
    /// The three widths of the cell
    Width(String),
    /// The volume of the cell
    Volume(String),
}

#[derive(Debug, AutoArgs, PartialEq)]
struct SquareWellParams {
    well_width: String,
    cell: Dimensions,
}

#[test]
fn craziness() {
    println!("help: {}", Params::help());
    assert!(Params::help().contains("resume-from"));

    assert_eq!(
        Params::ResumeFrom("hello".to_string()),
        Params::from_iter(&["", "resume-from", "hello"]).unwrap()
    );

    // `sys` is `_Params`'s only field, and `SquareWellParams` itself has
    // a single enum-typed field (`cell`), so both are keyword-less
    // defaults: `cell`'s own keyword (`width`/`volume`) is what
    // ultimately drives dispatch, with `well_width` as an ordinary flag
    // alongside it.
    assert_eq!(
        Params::_Params {
            sys: SquareWellParams {
                well_width: "1.3".to_string(),
                cell: Dimensions::Volume("5".to_string()),
            }
        },
        Params::from_iter(&["", "--well-width", "1.3", "volume", "5"]).unwrap()
    );

    assert_eq!(
        Params::_Params {
            sys: SquareWellParams {
                well_width: "1.3".to_string(),
                cell: Dimensions::Width("3x3x3".to_string()),
            }
        },
        Params::from_iter(&["", "width", "3x3x3", "--well-width", "1.3"]).unwrap()
    );
}
