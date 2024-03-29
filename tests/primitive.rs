// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate auto_args;

use auto_args::AutoArgs;

#[test]
fn simple_u64() {
    println!("help: {}", u64::help());
    assert!(!u64::help().contains("--first"));

    assert_eq!(7, u64::from_iter(&["", "7"]).unwrap());

    #[cfg(feature = "meval")]
    assert_eq!(7, u64::from_iter(&["", "7e0"]).unwrap());
    #[cfg(feature = "meval")]
    assert_eq!(1000000, u64::from_iter(&["", "1e6"]).unwrap());

    assert!(u64::from_iter(&["hello"]).is_err());
}

#[test]
fn simple_string() {
    println!("help: {}", String::help());
    assert!(!String::help().contains("--first"));

    assert_eq!("7".to_string(), String::from_iter(&["", "7"]).unwrap());

    assert!(String::from_iter(&[""]).is_err());
}

#[test]
fn simple_option_string() {
    println!("help: {}", <Option<String>>::help());
    assert!(!<Option<String>>::help().contains("--first"));

    assert_eq!(
        Some("7".to_string()),
        <Option<String>>::from_iter(&["", "7"]).unwrap()
    );

    assert_eq!(None, <Option<String>>::from_iter(&[""]).unwrap());
}

#[test]
fn simple_option_vec_i16() {
    println!("help: {}", <Vec<i16>>::help());
    println!("\nhelp is mysterious...\n");
    assert!(!<Vec<i16>>::help().contains("--first"));

    println!("getting a seven");
    assert_eq!(Ok(vec![7]), <Vec<i16>>::from_iter(&["", "7"]));

    println!("getting an empty list");
    assert_eq!(Vec::<i16>::new(), <Vec<i16>>::from_iter(&[""]).unwrap());
}

#[test]
fn simple_f64_many_ways() {
    println!("help: {}", <f64>::help());

    assert_eq!(0.3, <f64>::from_iter(&["", "0.3"]).unwrap());

    assert_eq!(7.0, <f64>::from_iter(&["", "7"]).unwrap());

    #[cfg(feature = "meval")]
    assert_eq!(1.0 / 3.0, <f64>::from_iter(&["", "1/3"]).unwrap());

    #[cfg(feature = "meval")]
    assert_eq!(3.0_f64.sqrt(), <f64>::from_iter(&["", "sqrt(3)"]).unwrap());

    #[cfg(feature = "meval")]
    assert_eq!(3.0_f64.sqrt(), <f64>::from_iter(&["", "3^(1/2)"]).unwrap());

    assert_eq!(1e300, <f64>::from_iter(&["", "1e300"]).unwrap());

    #[cfg(feature = "meval")]
    assert_eq!(
        1e300_f64.sqrt(),
        <f64>::from_iter(&["", "sqrt(1e300)"]).unwrap()
    );
}
