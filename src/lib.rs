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
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments) -> Result<Self, pico_args::Error>;
    /// Indicates whether this type requires any input.
    ///
    /// This is false if the data may be processed with no input, true
    /// otherwise.  There is a default implementation of `false` for
    /// convenience, since this is the "safe" answer.
    fn requires_input() -> bool {
        false
    }
    /// Return a tiny  help message.
    fn tiny_help_message(key: &'static str, name: &'static str) -> String;
    /// Return a help message.
    fn help_message(key: &'static str, name: &'static str, doc: &'static str) -> String {
        format!("    {}  {}", Self::tiny_help_message(key, name), doc)
    }
}

/// A list of possible errors.
#[derive(Clone, Debug)]
pub enum Error {
    /// An error from pico-args.
    Pico(pico_args::Error),

    /// A missing required flag.
    MissingOption(&'static str),
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
    fn parse_internal(key: &'static str, args: &mut pico_args::Arguments) -> Result<Self, pico_args::Error> {
        if key == "" {
            let copy = args.clone();
            let free = copy.free()?;
            if free.len() == 0 {
                Err(pico_args::Error::OptionWithoutAValue(""))
            } else {
                *args = pico_args::Arguments::from_args(free[1..].to_vec());
                Ok(free[0].clone())
            }
        } else {
            if let Some(a) = args.value_from_str(key)? {
                Ok(a)
            } else {
                Err(pico_args::Error::OptionWithoutAValue(key))
            }
        }
    }
    fn tiny_help_message(key: &'static str, name: &'static str) -> String {
        let name = if name == "" { "STRING" } else { name };
        if key == "" {
            name.to_string()
        } else {
            format!("{} {}", key, name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn should_parse<T: Eq + AutoArgs + std::fmt::Debug>(args: &'static [&'static str],
                                                        key: &'static str,
                                                        result: T) {
        let owned_args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
        let mut args = pico_args::Arguments::from_args(owned_args);
        assert_eq!(T::parse_internal(key, &mut args).unwrap(), result);
    }

    #[test]
    fn hello_world() {
        let flags = &["--hello", "world", "--bad"];
        should_parse(flags, "--hello", "world".to_string());
        should_parse(flags, "--hello", "world".to_string());
    }
    #[test]
    fn positional_arg() {
        let flags = &["bad"];
        should_parse(flags, "", "bad".to_string());
    }
}
