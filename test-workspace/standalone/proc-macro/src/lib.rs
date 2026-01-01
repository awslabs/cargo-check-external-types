// SPDX-License-Identifier: Apache-2.0

//! A proc-macro crate that should be skipped by the tool.

use proc_macro::TokenStream;

/// A simple derive macro for testing.
#[proc_macro_derive(TestDerive)]
pub fn test_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
