// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Regression tests for bugs found in code review of the subcommand-style
// enum rewrite: a struct with 2+ enum-typed fields (the legacy flag-style
// fallback) combined with a default variant whose fields are all
// optional, a failed subcommand attempt leaving `args` partially
// consumed, and a non-UTF-8 first argument being misreported.

use auto_args::AutoArgs;

/// An enum with a keyword-less default variant whose own field is
/// optional, so `Inner::REQUIRES_INPUT` used to become `false`.
#[derive(Debug, AutoArgs, PartialEq)]
enum Inner {
    Keyworded(String),
    _Default { opt: Option<String> },
}

#[derive(Debug, AutoArgs, PartialEq)]
enum Outer {
    V1 { inner: Inner },
    V2 { other: String },
}

/// Two enum-typed fields force the legacy flag-style fallback for both.
#[derive(Debug, AutoArgs, PartialEq)]
struct Holder {
    a: Outer,
    b: Outer,
}

#[test]
fn fallback_disambiguation_does_not_need_a_spurious_extra_flag() {
    // Before the fix, `Inner`'s `REQUIRES_INPUT` being (incorrectly)
    // computed from just its default variant made `check_main_flag`
    // demand a redundant bare `--a-v1` flag on top of the perfectly
    // unambiguous `--a-v1-inner-keyworded`.
    assert_eq!(
        Ok(Holder {
            a: Outer::V1 {
                inner: Inner::Keyworded("hi".to_string())
            },
            b: Outer::V2 {
                other: "yo".to_string()
            },
        }),
        Holder::from_iter(&[
            "",
            "--a-v1-inner-keyworded",
            "hi",
            "--b-v2-other",
            "yo"
        ])
    );
}

#[derive(Debug, AutoArgs, PartialEq)]
enum MyEnum {
    First { first: String },
}

#[derive(Debug, AutoArgs, PartialEq)]
struct Opt {
    thing: Option<MyEnum>,
}

#[test]
fn failed_subcommand_attempt_does_not_partially_consume_args() {
    // A typo (`--bogus` instead of `--first`) makes the `first` variant
    // fail to parse. Wrapped in `Option<_>`, that failure is swallowed,
    // but the keyword token itself must NOT have been silently consumed
    // by the failed attempt -- it should still show up as leftover,
    // unexpected input, rather than vanishing.
    match Opt::parse_vec(vec![
        "first".into(),
        "--bogus".into(),
        "x".into(),
    ]) {
        Err(auto_args::Error::UnexpectedOption(leftover)) => {
            assert!(
                leftover.contains("first"),
                "the recognized-but-then-failed keyword should still be visible \
                 in the leftover/unexpected arguments, got: {}",
                leftover
            );
        }
        other => panic!("expected UnexpectedOption, got {:?}", other),
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_first_argument_is_reported_as_invalid_not_missing() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    #[derive(Debug, AutoArgs, PartialEq)]
    enum Cmd {
        Hello { name: String },
        Goodbye { name: String },
    }

    let bad = OsString::from_vec(vec![0xFF, 0xFE]);
    let mut args = vec![bad];
    match Cmd::parse_internal("", &mut args) {
        Err(auto_args::Error::InvalidUTF8(_)) => {}
        other => panic!(
            "expected InvalidUTF8 for a non-UTF-8 first argument, got {:?}",
            other
        ),
    }
}
