// Copyright 2018 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate auto_args;

use auto_args::AutoArgs;

use std::ffi::OsString;

/// The parameters needed to configure a square well system.
#[derive(PartialEq, Debug, AutoArgs)]
struct Vector3d<T> {
    x: T,
    y: T,
    z: T,
}

/// A description of the cell dimensions
#[derive(PartialEq, Debug, AutoArgs)]
#[allow(non_snake_case)]
enum CellDimensions {
    /// The three widths of the cell
    CellWidth(Vector3d<f64>),
    /// The volume of the cell
    CellVolume(f64),
}

/// The parameters needed to configure a square well system.
#[derive(PartialEq, Debug, AutoArgs)]
struct SquareWellParams {
    well_width: f64,
    _dim: CellDimensions,
}

#[allow(non_snake_case)]
#[derive(PartialEq, Debug, AutoArgs)]
struct SadParams {
    /// The minimum temperature we are interested in.
    min_T: f64,
    /// The seed for the random number generator.
    seed: Option<u64>,
}

#[derive(PartialEq, Debug, AutoArgs)]
enum Params<MP, SP> {
    ResumeFrom(String),
    _Params { _sys: SP, _mc: MP },
}

#[derive(PartialEq, Debug, AutoArgs)]
struct Simple {
    simple: u64,
}

#[derive(PartialEq, Debug, AutoArgs)]
struct Naive {
    naive: String,
}

#[test]
fn craziness() {
    type P = Params<SadParams, SquareWellParams>;
    println!("help: {}", P::help());
    println!("\n\n\n\n");
    assert!(P::help().contains("--resume-from "));
    assert!(P::help().contains("--well-width "));
    assert!(P::help().contains("--cell-volume "));
    assert!(P::help().contains("--cell-width-x "));
    assert!(!P::help().contains("--dim- "));

    assert_eq!(
        Params::ResumeFrom::<SadParams, SquareWellParams>("hello".to_string()),
        P::parse_vec(vec![
            OsString::from("--resume-from"),
            OsString::from("hello")
        ])
        .unwrap()
    );

    assert_eq!(
        Params::ResumeFrom::<Naive, Simple>("hello".to_string()),
        Params::<Naive, Simple>::parse_vec(vec![
            OsString::from("--resume-from"),
            OsString::from("hello")
        ])
        .unwrap()
    );

    assert_eq!(
        Params::_Params::<Naive, Simple> {
            _sys: Simple { simple: 37 },
            _mc: Naive {
                naive: "goodbye".to_string(),
            },
        },
        Params::<Naive, Simple>::parse_vec(vec![
            OsString::from("--simple"),
            OsString::from("37"),
            OsString::from("--naive"),
            OsString::from("goodbye")
        ])
        .unwrap()
    );

    assert_eq!(
        Params::_Params::<SadParams, Simple> {
            _sys: Simple { simple: 137 },
            _mc: SadParams {
                min_T: 0.2,
                seed: None,
            },
        },
        Params::<SadParams, Simple>::parse_vec(vec![
            OsString::from("--simple"),
            OsString::from("137"),
            OsString::from("--min-T"),
            OsString::from("0.2")
        ])
        .unwrap()
    );

    assert_eq!(
        Params::_Params::<SadParams, SquareWellParams> {
            _sys: SquareWellParams {
                well_width: 1.3,
                _dim: CellDimensions::CellVolume(5.0),
            },
            _mc: SadParams {
                min_T: 0.2,
                seed: None,
            },
        },
        P::parse_vec(vec![
            OsString::from("--well-width"),
            OsString::from("1.3"),
            OsString::from("--cell-volume"),
            OsString::from("5"),
            OsString::from("--min-T"),
            OsString::from("0.2")
        ])
        .unwrap()
    );
}
