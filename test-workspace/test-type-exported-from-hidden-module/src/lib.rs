/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// no_std makes reading the RustDoc-generated JSON thousands of lines smaller, and therefore easier to
// manually debug.
#![no_std]

pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
// TODO: Because this public module is declared within a hidden module, it can't be checked for external types.
pub use hidden_module::public_module;

#[doc(hidden)]
mod hidden_module {
    pub struct InnerStructA;
    pub struct InnerStructB(pub external_lib::SimpleNewType);
    pub struct InnerStructC {
        pub inner: external_lib::SimpleNewType,
    }
    pub use external_lib::SimpleNewType;

    pub fn inner_fn() -> external_lib::SimpleNewType {}

    pub enum InnerEnum {
        SimpleNewType(SimpleNewType)
    }

    pub mod public_module {
        pub struct InnerStructD {
            pub inner: external_lib::SimpleNewType,
        }
    }
}
