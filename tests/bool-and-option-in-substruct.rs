// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[derive(Debug, AutoArgs, PartialEq, Eq)]
struct SubOpt {
    max_iter: Option<u64>,
    quiet: bool,
}

#[derive(Debug, AutoArgs, PartialEq, Eq)]
struct Opt {
    foo: String,
    _report: SubOpt,
}

#[derive(Debug, AutoArgs, PartialEq, Eq)]
enum Parent {
    CaseOne,
    _CaseTwo(Opt),
}

#[test]
fn just_opt() {
    println!("help: {}", Opt::help());
    assert!(Opt::help().contains("--quiet"));
    let parent = Opt::parse_vec(vec![std::ffi::OsString::from("--foo=foo")]).unwrap();
    assert_eq!(
        Opt {
            foo: "foo".to_string(),
            _report: SubOpt {
                max_iter: None,
                quiet: false,
            }
        },
        parent
    );
}

#[test]
fn craziness() {
    println!("help: {}", Parent::help());
    assert!(Parent::help().contains("--quiet"));
    let parent = Parent::parse_vec(vec![std::ffi::OsString::from("--foo=foo")]).unwrap();
    assert_eq!(
        Parent::_CaseTwo(Opt {
            foo: "foo".to_string(),
            _report: SubOpt {
                max_iter: None,
                quiet: false,
            }
        }),
        parent
    );
}
