#![deny(missing_docs)]

//! This crate enables you to create a command-line interface by
//! defining a struct to hold your options.

use std::ffi::OsString;
use std::path::PathBuf;

#[doc(hidden)]
pub use auto_args_derive::*;

/// The primary trait, which is implemented by any type which may be
/// part of your command-line flags.
pub trait AutoArgs: Sized {
    /// Parse the command-line arguments, exiting in case of error.
    ///
    /// This is what users actually use.
    fn parse_args() -> Self {
        let mut v: Vec<_> = std::env::args_os().collect();
        v.remove(0);
        if v.iter().any(|v| v == "--help") {
            println!("{}", Self::usage());
            std::process::exit(0);
        }
        match Self::parse_vec(v) {
            Ok(val) => val,
            Err(e) => {
                println!("error: {}\n", e);
                println!("{}", Self::usage());
                std::process::exit(1)
            }
        }
    }
    /// Parse a `Vec` of arguments as if they were command line flags
    ///
    /// This mimics what we would do if we were doing the real
    /// parsing, except that we don't exit on error.
    fn parse_vec(mut args: Vec<OsString>) -> Result<Self, Error> {
        let v = Self::parse_internal("", &mut args)?;
        if args.len() > 0 {
            Err(Error::UnexpectedOption(format!("{:?}", args)))
        } else {
            Ok(v)
        }
    }
    /// For implementation, but not for using this library.
    ///
    /// Parse this flag from the arguments, and return the set of
    /// remaining arguments if it was successful.  Otherwise return an
    /// error message indicating what went wrong.  The `prefix` is
    /// a string that should be inserted prior to a flag name.
    fn parse_internal(key: &str, args: &mut Vec<OsString>) -> Result<Self, Error>;
    /// Indicates whether this type requires any input.
    ///
    /// This is false if the data may be processed with no input, true
    /// otherwise.
    const REQUIRES_INPUT: bool;
    /// Return a tiny  help message.
    fn tiny_help_message(key: &str) -> String;
    /// Return a help message.
    fn help_message(key: &str, doc: &str) -> String {
        format!("    {}  {}", Self::tiny_help_message(key), doc)
    }
    /// Usage text for the actual command
    fn usage() -> String {
        format!("USAGE:
    {} {}

For more information try --help",
                std::env::args_os().next().unwrap().to_string_lossy(),
                Self::tiny_help_message(""))
    }
    /// Help text for the actual command
    fn help() -> String {
        format!("USAGE:
    {} {}

{}

For more information try --help",
                std::env::args_os().next().unwrap().to_string_lossy(),
                Self::tiny_help_message(""),
                Self::help_message("", ""))
    }
}

/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// An error from pico-args.
    OptionValueParsingFailed(String, String),

    /// A missing value from an option.
    InvalidUTF8(String),

    /// A missing value from an option.
    OptionWithoutAValue(String),

    /// A missing required flag.
    MissingOption(String),

    /// An unexpected option.
    UnexpectedOption(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::OptionValueParsingFailed(key,e) => {
                write!(f, "error parsing option '{}': {}", key, e)
            }
            Error::MissingOption(key) => {
                write!(f, "the required '{}' option is missing", key)
            }
            Error::InvalidUTF8(e) => {
                write!(f, "invalid UTF-8: '{}'", e)
            }
            Error::OptionWithoutAValue(key) => {
                write!(f, "the option '{}' is missing a value", key)
            }
            Error::UnexpectedOption(o) => {
                write!(f, "unexpected option: {}", o)
            }
        }
    }
}

impl std::error::Error for Error {}

macro_rules! impl_from_osstr {
    ($t:ty, $tyname:expr, $conv:expr) => {
        impl AutoArgs for $t {
            const REQUIRES_INPUT: bool = true;
            fn parse_internal(key: &str, args: &mut Vec<OsString>) -> Result<Self, Error> {
                let convert = $conv;
                if key == "" {
                    if args.len() == 0 {
                        Err(Error::OptionWithoutAValue("".to_string()))
                    } else {
                        let arg = if args[0] == "--" {
                            if args.len() > 1 {
                                args.remove(1)
                            } else {
                                return Err(Error::OptionWithoutAValue("".to_string()));
                            }
                        } else {
                            args.remove(0)
                        };
                        convert(arg)
                    }
                } else {
                    println!("looking for {:?} in {:?}", key, args);
                    let eqthing = format!("{}=", key);
                    if let Some(i) = args.iter().position(|v| v == key || v.to_string_lossy().starts_with(&eqthing)) {
                        let thing = args.remove(i)
                            .into_string()
                            .map_err(|e| Error::InvalidUTF8(format!("{:?}", e)))?;
                        println!("thing is {:?}", thing);
                        if thing == key {
                            if args.len() > i {
                                convert(args.remove(i))
                            } else {
                                Err(Error::OptionWithoutAValue(key.to_string()))
                            }
                        } else {
                            convert(thing.split_at(eqthing.len()).1.into())
                        }
                    } else {
                        Err(Error::MissingOption(key.to_string()))
                    }
                }
            }
            fn tiny_help_message(key: &str) -> String {
                if key == "" {
                    "STRING".to_string()
                } else {
                    format!("{} STRING", key)
                }
            }
        }

        impl AutoArgs for Vec<$t> {
            const REQUIRES_INPUT: bool = false;
            fn parse_internal(key: &str, args: &mut Vec<OsString>)
                              -> Result<Self, Error> {
                let mut res: Self = Vec::new();
                loop {
                    match <$t>::parse_internal(key, args) {
                        Ok(the_arg) => {
                            res.push(the_arg);
                        }
                        Err(Error::MissingOption(_)) => {
                            return Ok(res);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
            fn tiny_help_message(key: &str) -> String {
                if key == "" {
                    format!("{}...", $tyname)
                } else {
                    format!("{} {} ...", key, $tyname)
                }
            }
        }
    }
}

impl_from_osstr!(String, "STRING", |osstring: OsString| {
    osstring.into_string().map_err(|e| Error::InvalidUTF8(format!("{:?}", e)))
});
impl_from_osstr!(PathBuf, "PATH", |osstring: OsString| {
    Ok(osstring.into())
});

impl AutoArgs for bool {
    const REQUIRES_INPUT: bool = false;
    fn parse_internal(key: &str, args: &mut Vec<OsString>) -> Result<Self, Error> {
        if key == "" {
            if args.len() == 0 {
                Err(Error::OptionWithoutAValue("".to_string()))
            } else {
                if args[0] == "--" {
                    return Err(Error::OptionWithoutAValue("bool".to_string()));
                }
                let arg = args.remove(0);
                if arg == "false" {
                    Ok(false)
                } else if arg == "true" {
                    Ok(true)
                } else {
                    Err(Error::MissingOption("bool".to_string()))
                }
            }
        } else {
            println!("looking for {:?} in {:?}", key, args);
            Ok(args.iter().filter(|v| v.to_string_lossy() == key).next().is_some())
        }
    }
    fn tiny_help_message(key: &str) -> String {
        if key == "" {
            "STRING".to_string()
        } else {
            format!("{} STRING", key)
        }
    }
}

impl<T: AutoArgs> AutoArgs for Option<T> {
    const REQUIRES_INPUT: bool = false;
    fn parse_internal(key: &str, args: &mut Vec<OsString>) -> Result<Self, Error> {
        Ok(T::parse_internal(key, args).ok())
    }
    fn tiny_help_message(key: &str) -> String {
        format!("[{}]", T::tiny_help_message(key))
    }
}

macro_rules! impl_from {
    ($t:ty, $tyname:expr) => {
        impl AutoArgs for $t {
            const REQUIRES_INPUT: bool = true;
            fn parse_internal(key: &str, args: &mut Vec<OsString>)
                              -> Result<Self, Error> {
                use std::str::FromStr;
                let the_arg = String::parse_internal(key, args)?;
                match Self::from_str(&the_arg) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(Error::OptionValueParsingFailed(key.to_string(), e.to_string())),
                }
            }
            fn tiny_help_message(key: &str) -> String {
                if key == "" {
                    $tyname.to_string()
                } else {
                    format!("{} {}", key, $tyname)
                }
            }
        }

        impl AutoArgs for Vec<$t> {
            const REQUIRES_INPUT: bool = false;
            fn parse_internal(key: &str, args: &mut Vec<OsString>)
                              -> Result<Self, Error> {
                let mut res: Self = Vec::new();
                loop {
                    match <$t>::parse_internal(key, args) {
                        Ok(val) => {
                            res.push(val);
                        }
                        Err(Error::MissingOption(_)) => {
                            return Ok(res);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
            fn tiny_help_message(key: &str) -> String {
                if key == "" {
                    format!("{}...", $tyname.to_string())
                } else {
                    format!("{} {} ...", key, $tyname)
                }
            }
        }
    }
}

impl_from!(u8, "u8");
impl_from!(u16, "u16");
impl_from!(u32, "u32");
impl_from!(u64, "u64");
impl_from!(usize, "usize");

impl_from!(i8, "i8");
impl_from!(i16, "i16");
impl_from!(i32, "i32");
impl_from!(i64, "i64");
impl_from!(isize, "isize");

impl AutoArgs for f64 {
    const REQUIRES_INPUT: bool = true;
    fn parse_internal(key: &str, args: &mut Vec<OsString>)
                      -> Result<Self, Error> {
        let the_arg = String::parse_internal(key, args)?;
        meval::eval_str(the_arg)
            .map_err(|e| Error::OptionValueParsingFailed(key.to_string(), e.to_string()))
    }
    fn tiny_help_message(key: &str) -> String {
        if key == "" {
            "FLOAT".to_string()
        } else {
            format!("{} FLOAT", key)
        }
    }
}

impl AutoArgs for Vec<f64> {
    const REQUIRES_INPUT: bool = false;
    fn parse_internal(key: &str, args: &mut Vec<OsString>)
                      -> Result<Self, Error> {
        let mut res: Self = Vec::new();
        loop {
            match <f64>::parse_internal(key, args) {
                Ok(val) => {
                    res.push(val);
                }
                Err(Error::MissingOption(_)) => {
                    return Ok(res);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    fn tiny_help_message(key: &str) -> String {
        format!("{} ...", f64::tiny_help_message(key))
    }
}
impl AutoArgs for f32 {
    const REQUIRES_INPUT: bool = true;
    fn parse_internal(key: &str, args: &mut Vec<OsString>)
                      -> Result<Self, Error> {
        let the_arg = String::parse_internal(key, args)?;
        meval::eval_str(the_arg)
            .map(|v| v as f32)
            .map_err(|e| Error::OptionValueParsingFailed(key.to_string(), e.to_string()))
    }
    fn tiny_help_message(key: &str) -> String {
        f64::tiny_help_message(key)
    }
}

impl AutoArgs for Vec<f32> {
    const REQUIRES_INPUT: bool = false;
    fn parse_internal(key: &str, args: &mut Vec<OsString>)
                      -> Result<Self, Error> {
        let mut res: Self = Vec::new();
        loop {
            match <f32>::parse_internal(key, args) {
                Ok(val) => {
                    res.push(val);
                }
                Err(Error::MissingOption(_)) => {
                    return Ok(res);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    fn tiny_help_message(key: &str) -> String {
        Vec::<f64>::tiny_help_message(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate as auto_args;
    fn should_parse<T: PartialEq + AutoArgs + std::fmt::Debug>(args: &'static [&'static str],
                                                               key: &'static str,
                                                               result: T) {
        let mut args: Vec<_> = args.iter().map(|s| OsString::from(s)).collect();
        assert_eq!(T::parse_internal(key, &mut args).unwrap(), result);
    }
    fn should_parse_completely<T: PartialEq + AutoArgs + std::fmt::Debug>(args: &'static [&'static str],
                                                               key: &'static str,
                                                               result: T) {
        let mut args: Vec<_> = args.iter().map(|s| OsString::from(s)).collect();
        assert_eq!(T::parse_internal(key, &mut args).unwrap(), result);
        if args.len() != 0 {
            println!("args remaining: {:?}", args);
            assert_eq!(args.len(), 0);
        }
    }

    fn shouldnt_parse<T: PartialEq + AutoArgs + std::fmt::Debug>(args: &'static [&'static str],
                                                                 key: &'static str) {
        let mut args: Vec<_> = args.iter().map(|s| OsString::from(s)).collect();
        assert!(T::parse_internal(key, &mut args).is_err());
    }

    #[test]
    fn hello_world() {
        let flags = &["--hello", "world", "--bad"];
        should_parse(flags, "--hello", "world".to_string());
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u8>(flags, "--hello");
    }
    #[test]
    fn hello_world_complete() {
        let flags = &["--hello", "world"];
        should_parse_completely(flags, "--hello", "world".to_string());
    }
    #[test]
    fn hello_list() {
        let flags = &["--hello", "big", "--hello", "bad", "--hello", "wolf"];
        should_parse(flags, "--hello",
                     vec!["big".to_string(), "bad".to_string(), "wolf".to_string()]);
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u8>(flags, "--hello");
    }
    #[test]
    fn positional_arg() {
        let flags = &["bad"];
        should_parse(flags, "", "bad".to_string());
    }
    #[test]
    fn arg_u8() {
        let flags = &["--hello", "8", "--goodbye", "255", "--bad"];
        should_parse(flags, "--hello", 8u8);
        should_parse(flags, "--goodbye", 255u8);
        shouldnt_parse::<String>(flags, "--helloo");
    }
    #[test]
    fn arg_i32() {
        let flags = &["--hello", "-100008", "--goodbye", "255", "--bad"];
        should_parse(flags, "--hello", -100008i32);
        should_parse(flags, "--hello", -100008i64);
        should_parse(flags, "--goodbye", 255i32);
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u32>(flags, "--hello");
    }
    #[test]
    fn arg_equal_i32() {
        let flags = &["--hello=-100008", "--goodbye", "255", "--bad"];
        should_parse(flags, "--hello", -100008i32);
        should_parse(flags, "--hello", -100008i64);
        should_parse(flags, "--goodbye", 255i32);
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u32>(flags, "--hello");
    }
    #[test]
    fn arg_f64() {
        let flags = &["--hello=3e13", "--goodbye", "2^10", "--bad"];
        should_parse(flags, "--hello", 3e13);
        should_parse(flags, "--goodbye", 1024.0);
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u32>(flags, "--hello");
    }
    #[test]
    fn arg_pathbuf() {
        let flags = &["--hello=3e13", "--goodbye", "2^10", "--bad"];
        should_parse(flags, "--hello", PathBuf::from("3e13"));
        should_parse(flags, "--goodbye", PathBuf::from("2^10"));
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u32>(flags, "--hello");
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Test {
        a: String,
        b: String,
    }
    #[test]
    fn derive_test() {
        println!("help:\n{}", Test::help_message("", "this is the help"));
        println!("help prefix --foo:\n{}", Test::help_message("--foo", "this is the help"));
        let flags = &["--a=foo", "--b", "bar"];
        should_parse_completely(flags, "", Test { a: "foo".to_string(), b: "bar".to_string() });
        shouldnt_parse::<String>(flags, "--helloo");

        let foo_flags = &["--foo-a=foo", "--foo-b", "bar"];
        should_parse_completely(foo_flags, "--foo",
                                Test { a: "foo".to_string(), b: "bar".to_string() });
        shouldnt_parse::<Test>(foo_flags, "");
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    struct Pair<T> {
        first: T,
        second: T,
    }
    #[test]
    fn derive_test_pair() {
        println!("help:\n{}", Pair::<Test>::help_message("", "this is the help"));
        let flags = &["--first-a=a1", "--first-b", "b1",
                      "--second-a", "a2", "--second-b", "b2"];
        should_parse_completely(flags, "", Pair {
            first: Test { a: "a1".to_string(), b: "b1".to_string() },
            second: Test { a: "a2".to_string(), b: "b2".to_string() },
        });
        shouldnt_parse::<String>(flags, "--helloo");
        assert!(!Pair::<Option<String>>::REQUIRES_INPUT);
        assert!(Pair::<String>::REQUIRES_INPUT);
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    enum Either<A,B> {
        Left(A),
        Right(B),
    }
    #[test]
    fn derive_either() {
        let flags = &["--left", "37"];
        should_parse_completely(flags, "", Either::<u8,u16>::Left(37u8));
    }
    #[test]
    fn derive_pair_either() {
        let flags = &["--first-left", "37", "--second-right", "3"];
        should_parse_completely(flags, "", Pair {
            first: Either::Left(37),
            second: Either::Right(3),
        });
    }
    #[test]
    fn derive_either_either() {
        let flags = &["--right-left", "37"];
        should_parse_completely(flags, "", Either::<u32,Either<u8,u16>>::Right(Either::Left(37)));
    }
    #[test]
    fn derive_either_option() {
        let flags = &["--right-left", "7"];
        should_parse_completely(flags, "",
                                Either::<u32,Either<u8,Option<u32>>>::Right(Either::Left(7)));

        let flags = &["--right-right"];
        should_parse_completely(flags, "",
                                Either::<u32,Either<u8,Option<u32>>>::Right(Either::Right(None)));

        let flags = &["--right-right", "5"];
        should_parse_completely(flags, "",
                                Either::<u32,Either<u8,Option<u32>>>::Right(Either::Right(Some(5))));
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    enum MyEnum {
        Hello {
            foo: String,
            bar: u8,
        },
        _Goodbye {
            baz: String
        },
    }
    #[test]
    fn derive_myenum() {
        let flags = &["--hello-foo", "good", "--hello-bar", "7"];
        should_parse(flags, "", MyEnum::Hello {
            foo: "good".to_string(),
            bar: 7,
        });
    }
    #[test]
    fn option() {
        let flags = &["--foo", "good"];
        should_parse(flags, "--foo", Some("good".to_string()));
        should_parse(flags, "--bar", Option::<String>::None);
        assert!(String::REQUIRES_INPUT);
        assert!(!Option::<String>::REQUIRES_INPUT);
    }
    #[derive(AutoArgs, PartialEq, Debug)]
    struct TupleStruct(usize);
    #[test]
    fn tuple_struct() {
        let flags = &["--foo", "5"];
        should_parse_completely(flags, "--foo", TupleStruct(5));
    }
}
