use auto_args::AutoArgs;

#[derive(AutoArgs, PartialEq, Debug)]
enum Exclusive {
    First { a: String },
}

#[test]
fn craziness() {
    println!("help: {}", Exclusive::help());
    println!("\n\n\n\n");
    assert!(Exclusive::help().contains("--a "));
    assert!(Exclusive::help().contains("first"));

    assert!(Exclusive::from_iter(&[""]).is_err());

    assert_eq!(
        Exclusive::First {
            a: "hi".to_string()
        },
        Exclusive::from_iter(&["", "first", "--a", "hi"]).unwrap()
    );
}
