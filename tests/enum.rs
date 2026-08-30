// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[test]
fn simple_enum() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        First { first: String },
        Second { second: i32 },
        Third { third: u16 },
    }
    println!("help: {}", EnumOpt::help());
    // The keyword selects the variant; its own field is then an
    // unprefixed flag (here the field happens to share its name with
    // the keyword, which is legal, if a little confusing).
    assert!(EnumOpt::help().contains("first"));
    assert!(EnumOpt::help().contains("--first "));
    assert!(EnumOpt::help().contains("second"));
    assert!(EnumOpt::help().contains("--second "));
    assert!(EnumOpt::help().contains("third"));
    assert!(EnumOpt::help().contains("--third "));

    assert_eq!(
        EnumOpt::First {
            first: "hello".to_string()
        },
        EnumOpt::from_iter(&["", "first", "--first", "hello"]).unwrap()
    );

    assert_eq!(
        EnumOpt::Second { second: 5 },
        EnumOpt::from_iter(&["", "second", "--second", "5"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());

    assert!(EnumOpt::from_iter(&["", "typo"]).is_err());
}

#[test]
fn unit_enum() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        First,
        Second,
        Third,
    }
    println!("help: {}", EnumOpt::help());
    assert!(EnumOpt::help().contains("first"));
    assert!(EnumOpt::help().contains("second"));
    assert!(EnumOpt::help().contains("third"));

    assert_eq!(EnumOpt::First, EnumOpt::from_iter(&["", "first"]).unwrap());

    assert_eq!(
        EnumOpt::Second,
        EnumOpt::from_iter(&["", "second"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());

    // Only one keyword may appear; the second is left over and rejected.
    assert!(EnumOpt::from_iter(&["", "first", "second"]).is_err());
}

#[test]
fn unit_enum_with_underscores() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        First_World,
        /// The second option is awesome!
        Second,
        T_,
    }
    println!("help: {}", EnumOpt::help());
    assert!(EnumOpt::help().contains("First-World"));
    assert!(EnumOpt::help().contains("second"));
    assert!(EnumOpt::help().contains("The second option is awesome!"));
    assert!(EnumOpt::help().contains("T"));

    assert_eq!(
        EnumOpt::First_World,
        EnumOpt::from_iter(&["", "First-World"]).unwrap()
    );

    assert_eq!(
        EnumOpt::Second,
        EnumOpt::from_iter(&["", "second"]).unwrap()
    );

    assert_eq!(EnumOpt::T_, EnumOpt::from_iter(&["", "T"]).unwrap());

    assert!(EnumOpt::from_iter(&[""]).is_err());

    assert!(EnumOpt::from_iter(&["", "first", "second"]).is_err());
}

#[test]
fn enum_with_singular_tuple() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        /// The foo integer
        Foo(u32),
        /// The bar String
        Bar(String),
    }
    println!("help: {}", EnumOpt::help());
    assert!(EnumOpt::help().contains("foo"));
    assert!(EnumOpt::help().contains("bar"));
    assert!(EnumOpt::help().contains("The foo integer"));
    assert!(EnumOpt::help().contains("The bar String"));

    // The tuple variant's field is now a bare positional value, not a
    // `--foo=VALUE` flag.
    assert_eq!(
        EnumOpt::Foo(37),
        EnumOpt::from_iter(&["", "foo", "37"]).expect("Trouble right here")
    );

    assert_eq!(
        EnumOpt::Bar("hello".to_string()),
        EnumOpt::from_iter(&["", "bar", "hello"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());

    assert!(EnumOpt::from_iter(&["", "foo", "37", "bar", "hello"]).is_err());
}

#[test]
fn enum_with_underscore_variant() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        _Greet { hello: String },
        Goodbye(String),
    }
    println!("help: {}", EnumOpt::help());
    assert!(EnumOpt::help().contains("--hello "));
    assert!(EnumOpt::help().contains("goodbye"));

    // `_Greet` has no keyword: it is the keyword-less default variant.
    assert_eq!(
        EnumOpt::_Greet {
            hello: "David".to_string()
        },
        EnumOpt::from_iter(&["", "--hello", "David"]).unwrap()
    );

    assert_eq!(
        EnumOpt::Goodbye("David".to_string()),
        EnumOpt::from_iter(&["", "goodbye", "David"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());
}

#[test]
fn enum_with_nested_underscore_variant() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum EnumOpt {
        _Greet { hello: String },
        Goodbye(String),
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Opt {
        say: EnumOpt,
    }
    println!("help: {}", Opt::help());
    // `say` is the struct's sole enum-typed field, so it becomes a
    // subcommand rather than a `--say-...`-prefixed flag: no prefix
    // leaks through to the enum's own keyword/fields.
    assert!(Opt::help().contains("--hello "));
    assert!(Opt::help().contains("goodbye"));

    assert_eq!(
        Opt {
            say: EnumOpt::_Greet {
                hello: "David".to_string()
            }
        },
        Opt::from_iter(&["", "--hello", "David"]).unwrap()
    );

    assert_eq!(
        Opt {
            say: EnumOpt::Goodbye("David".to_string())
        },
        Opt::from_iter(&["", "goodbye", "David"]).unwrap()
    );

    assert!(EnumOpt::from_iter(&[""]).is_err());
}

#[test]
fn enum_missing_flag() {
    #[derive(AutoArgs, PartialEq, Debug)]
    enum Nested {
        Hello { hello: String, greeting: String },
        Goodbye(String),
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    enum Opt {
        Other(Nested),
        Nested(Nested),
    }
    println!("help: {}", Opt::help());
    assert!(Opt::help().contains("other"));
    assert!(Opt::help().contains("nested"));
    assert!(Opt::help().contains("goodbye"));
    assert!(Opt::help().contains("hello"));

    assert_eq!(
        Ok(Opt::Other(Nested::Hello {
            hello: "1".to_string(),
            greeting: "ee".to_string(),
        })),
        Opt::from_iter(&["", "other", "hello", "--hello", "1", "--greeting", "ee"])
    );

    assert!(Opt::from_iter(&[""]).is_err());

    assert_eq!(
        Ok(Opt::Other(Nested::Goodbye("World".to_string()))),
        Opt::from_iter(&["", "other", "goodbye", "World"])
    );

    assert_eq!(
        Err(auto_args::Error::MissingOption("--greeting".to_string())),
        Opt::from_iter(&["", "other", "hello", "--hello", "World"])
    );

    // `from_iter` discards its first element as a program name, so this
    // leaves no arguments at all.
    assert!(matches!(
        Opt::from_iter(&["--typo"]),
        Err(auto_args::Error::MissingSubcommand(_))
    ));

    // Here the (discarded) program name leaves "World" as the sole
    // remaining argument, which is not a known subcommand keyword.
    assert!(matches!(
        Opt::from_iter(&["--nested-hello-hello", "World"]),
        Err(auto_args::Error::UnknownSubcommand(w, _)) if w == "World"
    ));
}
