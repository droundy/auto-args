// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use auto_args::AutoArgs;

#[test]
fn simple_doc_comment() {
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Opt {
        /// First line
        /// Second line
        arg: u64,
    }
    println!("help: {}", Opt::help());
    assert!(
        Opt::help().contains("First line"),
        "Should use the first line from the doc comment"
    );
    assert!(
        !Opt::help().contains("Second line"),
        "Should not use the second line from the doc comment"
    );
}
