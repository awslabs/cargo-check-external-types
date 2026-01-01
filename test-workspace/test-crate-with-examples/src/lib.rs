// SPDX-License-Identifier: Apache-2.0

//! A test crate with examples to verify --lib flag works correctly.

use external_lib::SomeStruct;

/// A function that uses an external type.
pub fn uses_external_type(_s: &SomeStruct) {}
