//! # A user's guide for auto_args.
//! 
//! AutoArgs allows you to parse command line arguments by defining a
//! struct.  It combines [clap](https://crates.io/crates/clap) with
//! custom derive.
//! 
//! The basic idea is that you define a type that represents the
//! information you want on the command-line from the person running
//! your program, and `derive(AutoArgs)` on that type, and then call
//! `YourType::from_args()` to find out what your user gave you.
//! To begin with, let's look at an example of how you might actually
//! use `AutoArgs` in a real program.
//! 
//! ```should_panic
//! #[macro_use]
//! extern crate auto_args;
//! 
//! use std::path::PathBuf;
//! use auto_args::AutoArgs;
//! 
//! #[derive(Debug, AutoArgs)]
//! struct Opt {
//! /// Filling fraction
//! filling_fraction: f64,
//! /// Number of atoms
//! N: u32,
//! /// Output directory, working directory if not present
//! dir: Option<PathBuf>,
//! /// Activate verbose printing
//! verbose: bool,
//! }
//! 
//! fn main() {
//! let opt = Opt::from_args();
//! println!("{:?}", opt);
//! }
//! ```
//! The remainder of this guide will give examples of how the
//! command-line flags are constructed from your type, starting with
//! simple cases and moving gradually to more complex ones.  In no
//! case does your code involve more than defining a type, deriving
//! `AutoArgs` and calling the `from_args` method of your type.
//! I want to note that `AutoArgs` *almost always* produces long flags.
//! This is because I feel that long flags are generally the easiest
//! to use.  If you want to fine-tune your command-line interface,
//! `AutoArgs` may not be for you.
//! ## Just a flag
//! Most often, you will define a struct as your type.  We'll
//! start out with the simplest case, which is a single boolean
//! field within that struct.
//! ```
//! struct Foo {
//!     foo: bool,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  [--foo]
//! 
//! For more information try --help
//! ```
//! and the following help message.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  [--foo]
//! 
//!   [--foo] 
//! 
//! 
//! For more information try --help
//! ```
//! A single boolean flag is treated as an optional flag.
//! ## Adding help information
//! We add help information simply by adding ordinary doc comments
//! to our struct.
//! ```
//! struct Help {
//!     /// Print excess messages.
//!     verbose: bool,
//!     /// The temperature.
//!     T: bool,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  [--verbose] [--T]
//! 
//! For more information try --help
//! ```
//! and the following help message.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  [--verbose] [--T]
//! 
//!   [--verbose] Print excess messages.
//!   [--T]       The temperature.
//! 
//! 
//! For more information try --help
//! ```
//! I would always documentation for actual projects, so I'll try
//! to model that here, even though these examples are all
//! fictitious.
//! ## How the flag is determined
//! We saw above that the flag just had `--` prepended to the
//! field name.  The rule in general is only slightly more
//! complicated: every underscore is replaced with a `-`.
//! ```
//! struct Flags {
//!     /// a simple word has "--" prepended to it.
//!     verbose: bool,
//!     /// Underscores are replaced with "-" ...
//!     blue_is_nice_: bool,
//!     /// and capital letters are preserved.
//!     min_T: bool,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  [--verbose] [--blue-is-nice-] [--min-T]
//! 
//!   [--verbose]       a simple word has "--" prepended to it.
//!   [--blue-is-nice-] Underscores are replaced with "-" ...
//!   [--min-T]         and capital letters are preserved.
//! 
//! 
//! For more information try --help
//! ```
//! Thus you can create most any flag name you care for, and it is
//! easy to tell which flag corresponds to which field in your
//! struct.
//! ## Other types
//! You can add most standard library types to your struct,
//! basically anything that can be read or parsed from a `&str`.
//! I'd recommend sticking to owned types.
//! ```
//! struct Types {
//!     /// The name of the type
//!     name: String,
//!     /// The temperature of the type
//!     T: f64,
//!     /// The place where it is
//!     directory: std::path::PathBuf,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  --name STRING --T FLOAT --directory STRING
//! 
//!   --name STRING      The name of the type
//!   --T FLOAT          The temperature of the type
//!   --directory STRING The place where it is
//! 
//! 
//! For more information try --help
//! ```
//! I should note that integer types do allow their value to be
//! specified using scientific notation, as in `1e6` rather than
//! `1000000`.  This is in different from rust's `FromStr`
//! implementation.  AutoArgs does reject floating point values that
//! cannot be reversibly converted to the integer type that is
//! requested.
//! 
//! Furthermore, when providing numerical user input, users may
//! specify an expression such as `1/3` or `sqrt(2)`.  This is
//! most useful for floating point input where makes it easier to
//! give high-precision input when needed, but may also be helpful
//! for integers.
//! ## Optional flags
//! In the previous examples, every flag (except a `bool` flag)
//! was required to be specified by the user.  If you want a flag
//! to be optional, you just use the standard `Option` type.
//! ```
//! struct Optional {
//!     /// The name is an optional argument.
//!     name: Option<String>,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7  [--name STRING]
//! 
//!   [--name STRING] The name is an optional argument.
//! 
//! 
//! For more information try --help
//! ```
//! The value is then `None` if the user did not specify that flag.
//! ## Subcommands
//! If you want to offer several mutually exclusive ways of running
//! your program -- the way `git` or `gh` offer subcommands like
//! `git remote add` or `gh pr create` -- you use an `enum` (just as
//! always, in rust). A bare keyword, derived from the variant's
//! name, selects the variant; that variant's own fields then become
//! ordinary (unprefixed) flags of their own.
//! ```
//! enum Exclusive {
//!     /// In this context, the doc comment for the variant is not
//!     /// used by auto_args.
//!     First {
//!         /// This is the "a" value
//!         a: String,
//!         /// Only the first line of comment shows up in help.
//!         /// This is the "b" value, which you cannot specify unless
//!         /// you also select the `first` subcommand.
//!         b: String,
//!     },
//!     /// A string that must be given right after its own keyword
//!     SecondFlag(String),
//!     /// A flag with no value, and with a capital letter.
//!     Third_,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7 <first|second-flag|Third>
//! 
//! For more information try --help
//! ```
//! and the following help message.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7 <first|second-flag|Third>
//! 
//!   SUBCOMMANDS 	
//!   first        --a STRING --b STRING	In this context, the doc comment for the variant is not
//!   --a STRING  This is the "a" value
//!   --b STRING  Only the first line of comment shows up in help.
//!   second-flag STRING	A string that must be given right after its own keyword
//!   STRING      	Third		A flag with no value, and with a capital letter.
//! 
//! 
//! For more information try --help
//! ```
//! This example illustrates the three kinds of `enum` variants: a
//! `first --a STRING --b STRING` subcommand with its own flags, a
//! `second-flag STRING` subcommand taking a single positional value,
//! and a bare `Third` subcommand (capitalized, since its name has no
//! non-underscore lowercase letters to convert) with no value at
//! all. Note that you cannot use a tuple variant with more than one
//! field.
//! Note that the rules for constructing a variant's keyword are more
//! complicated than for struct fields. This is because by convention
//! variants are given `CamelCase` names, which aren't suitable as
//! keywords. If a variant name contains an underscore, then it is
//! treated like a field name (as described above), with any trailing
//! underscores removed. Otherwise the name is converted from
//! `CamelCase` to `kebab-case`. A variant whose name is *entirely*
//! underscores (so its keyword would be empty) is instead treated as
//! a keyword-less default: it is tried when no other variant's
//! keyword is present, so at most one variant of an enum may be
//! named this way.
//! If a struct has more than one field whose type is itself an
//! `enum` derived this way, there is no single sensible position for
//! two subcommand keywords, so all of that struct's enum-typed
//! fields fall back to ordinary prefixed flags instead, exactly as
//! enums used to work throughout this crate.
//! ## Nesting types
//! You can use any `AutoArgs` type as a field within a struct or
//! enum.  Doing so will give flag names that combine the nested
//! field names.
//! ```ignore
//! #[derive(AutoArgs)]
//! /// I'm not putting doc-comments on `x` and `y`, because auto_args
//! /// would give the same help message for `--position-x` as for
//! /// `--velocity-x`, which would be pretty useless.
//! struct Vec2d {
//!     x: f64,
//!     y: f64,
//! }
//! #[derive(AutoArgs)]
//! struct Nested {
//!     /// We would like for this to be the help for both components
//!     /// of the position, but auto_args is not that clever.  Ideally
//!     /// the help should read something like: the x component of
//!     /// the position/velocity, but that would require combining
//!     /// information at multiple levels and sounds hard.
//!     position: Vec2d,
//!     velocity: Vec2d,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7   --position-x FLOAT --position-y FLOAT  --velocity-x FLOAT --velocity-y FLOAT
//! 
//!   --position-x FLOAT 
//!   --position-y FLOAT 
//!   --velocity-x FLOAT 
//!   --velocity-y FLOAT 
//! 
//! 
//! For more information try --help
//! ```
//! ## Flattened nesting types
//! As you say in the last example, nesting types allows you to
//! make your own complex types that can be reused.  Sometimes,
//! however, you would like to nest structs for a different
//! reason: to separate concerns in the code.  In this case, you
//! may not want the nesting to be visible in the user interface.
//! This can be acheived with a leading underscore on a field
//! name.  The catch is that when you do this, you could run into
//! a runtime error if you have duplicate field names.
//! ```ignore
//! #[derive(AutoArgs)]
//! struct MyConfig {
//!     /// The user's name
//!     name: String,
//! }
//! #[derive(AutoArgs)]
//! struct YourConfig {
//!     /// The user's address
//!     address: String,
//! }
//! #[derive(AutoArgs)]
//! struct Flattened {
//!     _mine: MyConfig,
//!     _yours: YourConfig,
//! }
//! ```
//! This gives the following usage.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7   --name STRING  --address STRING
//! 
//! For more information try --help
//! ```
//! and the following help message.
//! ```ignore
//! USAGE:
//!   create_guide-9bb8923c28882ab7   --name STRING  --address STRING
//! 
//!   --name STRING    The user's name
//!   --address STRING The user's address
//! 
//! 
//! For more information try --help
//! ```
//! This may be a good idea if `MyConfig` and `YourConfig` are
//! implementation details that your user need not be aware of.
//! ## Other possibilities
//! There may be a few other features that auto_args has, for which I
//! have not bothered to create an entire example.  I will list
//! them here when they come to mind.
//! 1. You can use a `Vec<T>` for many values of `T` to create an
//! option that can be specified more than once.
//! ## Conclusion
//! There is more that could be said and more possible examples,
//! but I think this is enough to get you started using `AutoArgs`.
//! The intent is that any reasonable type that *can* be obtained
//! from one or more strings should work with auto_args.  Please fill
//! an issue on github if there is a type that you would like to
//! have supported by auto_args.  Pull requests are most welcome.
