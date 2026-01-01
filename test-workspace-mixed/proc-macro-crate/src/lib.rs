// SPDX-License-Identifier: Apache-2.0

//! A proc-macro crate in the mixed workspace.

use proc_macro::TokenStream;

/// A simple derive macro.
#[proc_macro_derive(MixedDerive)]
pub fn mixed_derive(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
