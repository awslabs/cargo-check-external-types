/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// no_std makes reading the RustDoc-generated JSON thousands of lines smaller, and therefore easier to
// manually debug.
#![no_std]

//! This crate exports a bunch of types for testing cargo-check-external-types against `test-crate`

pub struct SomeStruct;
pub struct SomeOtherStruct;

pub trait SimpleTrait {
    fn something(&self) -> u32;
}

impl SimpleTrait for () {
    fn something(&self) -> u32 {
        0
    }
}

pub trait SimpleGenericTrait<T> {
    fn something(&self, thing: T) -> u32;
}

pub trait AssociatedGenericTrait {
    type Input;
    type Output;
    type Error;

    fn something(&self, input: Self::Input) -> Self::Output;

    fn something_result(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

pub struct SimpleNewType(pub u32);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ReprCType {
    i: i32,
    f: f32,
}
