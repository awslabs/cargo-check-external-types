/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub(crate) const NEW_ISSUE_URL: &str =
    "https://github.com/awslabs/cargo-check-external-types/issues/new";

pub mod cargo;
pub mod config;
pub mod error;
pub mod path;
pub mod visitor;

/// A macro for attaching info to error messages pointing to the line of code responsible for the error.
/// [Thanks to dtolnay for this macro](https://github.com/dtolnay/anyhow/issues/22#issuecomment-542309452)
#[macro_export]
macro_rules! here {
    () => {
        concat!("error at ", file!(), ":", line!(), ":", column!())
    };
    ($($args:tt)+) => {
        format!("{} ({})", format!($($args)+), here!())
    };
}

/// Macro that indicates there is a bug in the program, but doesn't panic.
#[macro_export]
macro_rules! bug {
    ($($args:tt)+) => {
        {
            use owo_colors::{OwoColorize, Stream};
            eprint!("{}",
                "BUG: "
                    .if_supports_color(Stream::Stdout, |text| text.purple())
                    .if_supports_color(Stream::Stdout, |text| text.bold())
            );
            eprint!($($args)+);
            eprintln!(" This is a bug. Please report it with a piece of Rust code that triggers it at: {}", $crate::NEW_ISSUE_URL);
        }
    };
}

/// Macro that indicates there is a bug in the program and then panics.
#[macro_export]
macro_rules! bug_panic {
    ($($args:tt)+) => {
        $crate::bug!($($args)+);
        panic!("execution cannot continue");
    };
}
