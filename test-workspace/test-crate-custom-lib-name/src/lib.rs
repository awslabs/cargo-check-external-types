/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![feature(generic_associated_types)]
#![allow(dead_code)]

//! This crate is used to test cargo-check-external-types against a crate that
//! defines a custom lib name. It only offers a subset of possible violations,
//! see `test-crate` for a more complete example.

use external_lib::{SimpleNewType, SomeStruct};

pub static SOME_STRUCT: SomeStruct = SomeStruct;
pub const SOME_CONST: SomeStruct = SomeStruct;

pub mod some_pub_mod {
    use external_lib::SomeStruct;

    pub static OPTIONAL_STRUCT: Option<SomeStruct> = None;
    pub const OPTIONAL_CONST: Option<SomeStruct> = None;
}

pub type NotExternalReferencing = u32;
pub type ExternalReferencingTypeAlias = SomeStruct;
pub type OptionalExternalReferencingTypeAlias = Option<SomeStruct>;
pub type ExternalReferencingRawPtr = *const SomeStruct;

pub struct AssocConstStruct;

impl AssocConstStruct {
    pub const SOME_CONST: u32 = 5;

    pub const OTHER_CONST: SimpleNewType = SimpleNewType(5);
}
