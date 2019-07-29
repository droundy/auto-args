#[macro_use]
extern crate auto_args;

use auto_args::AutoArgs;

#[derive(AutoArgs, PartialEq, Debug)]
enum Exclusive {
    First {
        a: String,
    },
}

#[test]
fn craziness() {
    println!("help: {}", Exclusive::help());
    println!("\n\n\n\n");
    assert!(Exclusive::help().contains("--first-a "));

    assert!(Exclusive::help().contains("--first-a "));

    assert!(Exclusive::from_iter(&[""]).is_err());
}
