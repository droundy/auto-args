// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[test]
fn simple_phantom() {
    #[derive(AutoArgs, PartialEq, Debug)]
    struct PhantomOpt<T> {
        first: std::marker::PhantomData<T>,
        second: String,
    }
    println!("help: {}", <PhantomOpt<i32>>::help());
    assert!(!<PhantomOpt<i32>>::help().contains("--first"));
    assert!(<PhantomOpt<i32>>::help().contains("--second"));

    assert_eq!(
        PhantomOpt::<i32> {
            first: std::marker::PhantomData,
            second: "hello".to_string()
        },
        <PhantomOpt<i32>>::from_iter(&["", "--second=hello"]).unwrap()
    );

    assert!(<PhantomOpt<i32>>::from_iter(&[""]).is_err());
}
