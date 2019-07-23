#![deny(missing_docs)]

//! This crate enables you to create a command-line interface by
//! defining a struct to hold your options.

/// The primary trait, which is implemented by any type which may be
/// part of your command-line flags.
pub trait AutoArgs: Sized {
    /// Parse this flag from the arguments, and return the set of
    /// remaining arguments if it was successful.  Otherwise return an
    /// error message indicating what went wrong.  The `prefix` is
    /// a string that should be inserted prior to a flag name.
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments) -> Result<Self, Error>;
    /// Indicates whether this type requires any input.
    ///
    /// This is false if the data may be processed with no input, true
    /// otherwise.  There is a default implementation of `false` for
    /// convenience, since this is the "safe" answer.
    fn requires_input() -> bool {
        false
    }
    /// Return a tiny  help message.
    fn tiny_help_message(key: &'static str) -> String;
    /// Return a help message.
    fn help_message(key: &'static str, doc: &'static str) -> String {
        format!("    {}  {}", Self::tiny_help_message(key), doc)
    }
}

/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// An error from pico-args.
    Pico(pico_args::Error),

    /// A missing required flag.
    MissingOption(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Pico(e) => {
                write!(f, "{}", e)
            }
            Error::MissingOption(key) => {
                write!(f, "the required '{}' option is missing", key)
            }
        }
    }
}

impl std::error::Error for Error {}
impl From<pico_args::Error> for Error {
    fn from(e: pico_args::Error) -> Self {
        Error::Pico(e)
    }
}

impl AutoArgs for String {
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments) -> Result<Self, Error> {
        if key == "" {
            let copy = args.clone();
            let free = copy.free()?;
            if free.len() == 0 {
                Err(Error::Pico(pico_args::Error::OptionWithoutAValue("")))
            } else {
                *args = pico_args::Arguments::from_args(free[1..].to_vec());
                Ok(free[0].clone())
            }
        } else {
            if let Some(a) = args.value_from_str(key)? {
                Ok(a)
            } else {
                Err(Error::MissingOption(key.to_string()))
            }
        }
    }
    fn tiny_help_message(key: &'static str) -> String {
        if key == "" {
            "STRING".to_string()
        } else {
            format!("{} STRING", key)
        }
    }
}

impl AutoArgs for Vec<String> {
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
                      -> Result<Self, Error> {
        let mut res: Self = Vec::new();
        loop {
            match String::parse_internal(key, args) {
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
    fn tiny_help_message(key: &'static str) -> String {
        if key == "" {
            "STRING...".to_string()
        } else {
            format!("{} STRING ...", key)
        }
    }
}

macro_rules! impl_from {
    ($t:ty, $tyname:expr) => {
        impl AutoArgs for $t {
            fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
                              -> Result<Self, Error> {
                use std::str::FromStr;
                let the_arg = String::parse_internal(key, args)?;
                match Self::from_str(&the_arg) {
                    Ok(val) => Ok(val),
                    Err(e) => Err(Error::Pico(pico_args::Error::OptionValueParsingFailed(key, e.to_string()))),
                }
            }
            fn tiny_help_message(key: &'static str) -> String {
                if key == "" {
                    $tyname.to_string()
                } else {
                    format!("{} {}", key, $tyname)
                }
            }
        }

        impl AutoArgs for Vec<$t> {
            fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
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
            fn tiny_help_message(key: &'static str) -> String {
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
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
                      -> Result<Self, Error> {
        let the_arg = String::parse_internal(key, args)?;
        meval::eval_str(the_arg)
            .map_err(|e| Error::Pico(pico_args::Error::OptionValueParsingFailed(key, e.to_string())))
    }
    fn tiny_help_message(key: &'static str) -> String {
        if key == "" {
            "FLOAT".to_string()
        } else {
            format!("{} FLOAT", key)
        }
    }
}

impl AutoArgs for Vec<f64> {
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
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
    fn tiny_help_message(key: &'static str) -> String {
        format!("{} ...", f64::tiny_help_message(key))
    }
}
impl AutoArgs for f32 {
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
                      -> Result<Self, Error> {
        let the_arg = String::parse_internal(key, args)?;
        meval::eval_str(the_arg)
            .map(|v| v as f32)
            .map_err(|e| Error::Pico(pico_args::Error::OptionValueParsingFailed(key, e.to_string())))
    }
    fn tiny_help_message(key: &'static str) -> String {
        f64::tiny_help_message(key)
    }
}

impl AutoArgs for Vec<f32> {
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
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
    fn tiny_help_message(key: &'static str) -> String {
        Vec::<f64>::tiny_help_message(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    fn should_parse<T: PartialEq + AutoArgs + std::fmt::Debug>(args: &'static [&'static str],
                                                               key: &'static str,
                                                               result: T) {
        let owned_args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
        let mut args = pico_args::Arguments::from_args(owned_args);
        assert_eq!(T::parse_internal(key, &mut args).unwrap(), result);
    }

    fn shouldnt_parse<T: PartialEq + AutoArgs + std::fmt::Debug>(args: &'static [&'static str],
                                                                 key: &'static str) {
        let owned_args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
        let mut args = pico_args::Arguments::from_args(owned_args);
        assert!(T::parse_internal(key, &mut args).is_err());
    }

    #[test]
    fn hello_world() {
        let flags = &["--hello", "world", "--bad"];
        should_parse(flags, "--hello", "world".to_string());
        should_parse(flags, "--hello", "world".to_string());
        shouldnt_parse::<String>(flags, "--helloo");
        shouldnt_parse::<u8>(flags, "--hello");
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
}
