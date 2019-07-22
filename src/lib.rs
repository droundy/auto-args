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
                Err(Error::MissingOption(key))
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

macro_rules! impl_from {
    ($t:ty, $tyname:expr) => {
        impl AutoArgs for $t {
            fn parse_internal(key: &'static str, args: &mut pico_args::Arguments)
                              -> Result<Self, pico_args::Error> {
                let conflicts: Vec<_> = info.conflicted_flags.iter().map(AsRef::as_ref).collect();
                let ruo: Vec<_> = info.required_unless_one.iter().map(AsRef::as_ref).collect();
                if info.name == "" {
                    f(app.arg(clap::Arg::with_name(info.name)
                              .takes_value(true)
                              .value_name($tyname)
                              .requires_all(info.required_flags)
                              .required(info.required)
                              .help(&info.help)
                              .validator(|s| Self::from_str(&s).map(|_| ())
                                         .map_err(|e| e.to_string()))))
                } else if ruo.len() > 0 {
                    f(app.arg(clap::Arg::with_name(info.name)
                              .long(info.name)
                              .takes_value(true)
                              .value_name($tyname)
                              .requires_all(info.required_flags)
                              .required(info.required)
                              .conflicts_with_all(&conflicts)
                              .required_unless_one(&ruo)
                              .help(&info.help)
                              .validator(|s| Self::from_str(&s).map(|_| ())
                                         .map_err(|e| e.to_string()))))
                } else {
                    f(app.arg(clap::Arg::with_name(info.name)
                              .long(info.name)
                              .takes_value(true)
                              .value_name($tyname)
                              .requires_all(info.required_flags)
                              .required(info.required)
                              .conflicts_with_all(&conflicts)
                              .help(&info.help)
                              .validator(|s| Self::from_str(&s).map(|_| ())
                                         .map_err(|e| e.to_string()))))
                }
            }
            fn from_clap(name: &str, matches: &clap::ArgMatches) -> Option<Self> {
                // println!("from {} {:?}", name, matches.value_of(name));
                matches.value_of(name).map(|s| Self::from_str(s).unwrap())
            }
        }

        // impl ClapMe for Vec<$t> {
        //     fn with_clap<TT>(info: ArgInfo, app: clap::App,
        //                      f: impl FnOnce(clap::App) -> TT) -> TT {
        //         let conflicts: Vec<_> = info.conflicted_flags.iter().map(AsRef::as_ref).collect();
        //         if info.name == "" {
        //             f(app.arg(clap::Arg::with_name(info.name)
        //                       .takes_value(true)
        //                       .value_name($tyname)
        //                       .required(false)
        //                       .requires_all(info.required_flags)
        //                       .multiple(true)
        //                       .help(&info.help)
        //                       .validator(|s| <$t>::from_str(&s).map(|_| ())
        //                                  .map_err(|_| "oops".to_owned()))))
        //         } else {
        //             f(app.arg(clap::Arg::with_name(info.name)
        //                       .long(info.name)
        //                       .takes_value(true)
        //                       .value_name($tyname)
        //                       .required(false)
        //                       .requires_all(info.required_flags)
        //                       .conflicts_with_all(&conflicts)
        //                       .multiple(true)
        //                       .help(&info.help)
        //                       .validator(|s| <$t>::from_str(&s).map(|_| ())
        //                                  .map_err(|_| "oops".to_owned()))))
        //         }
        //     }
        //     fn from_clap(name: &str, matches: &clap::ArgMatches) -> Option<Self> {
        //         Some(matches.values_of(name).unwrap_or(clap::Values::default())
        //              .map(|s| <$t>::from_str(s).unwrap()).collect())
        //     }
        //     fn requires_flags(_name: &str) -> Vec<String> {
        //         vec![]
        //     }
        // }
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
