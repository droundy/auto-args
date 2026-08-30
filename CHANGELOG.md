* 0.4.0 - August 29 2026

    - **Breaking:** `enum`s derived with `AutoArgs` now parse like `git`/`gh`
      subcommands instead of flat mutually-exclusive flags. A bare keyword
      (from the variant's name) selects the variant, and that variant's own
      fields become unprefixed flags (e.g. `mytool hello --hello foo` instead
      of `mytool --hello-hello foo`). Chained/nested enums compose naturally
      (`mytool remote add name url`).
    - If a struct has two or more fields whose type is itself a derived
      `enum`, there is no single sensible subcommand position, so all of
      that struct's enum-typed fields fall back to the previous flag-style
      behavior.
    - Added `Error::UnknownSubcommand` and `Error::MissingSubcommand` for the
      new dispatch's error cases.
    - An enum variant whose fields are all optional no longer needs an extra
      bare flag to disambiguate "selected with nothing supplied" from
      "nothing supplied at all" -- the subcommand keyword itself is now that
      disambiguation.
    - **Breaking:** a single-field tuple variant's value is now given as a
      separate positional token after the keyword (`mytool foo 37`); the old
      combined `--foo=37` syntax no longer works for enum subcommands.
    - Two variants whose names produce the same keyword (e.g. `ABC` and `Abc`,
      both `abc`) is now a compile-time error, instead of silently making one
      of them unreachable.
    - A non-UTF-8 first argument is now reported as `Error::InvalidUTF8`
      rather than being misreported as a missing subcommand.

* 0.3.2 - June 24 2025

    - Bumped version of `syn` and related dependencies.
