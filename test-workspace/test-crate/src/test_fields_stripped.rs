/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// Stripped fields are any public fields marked as `#[doc(hidden)]`
pub struct SomeStructWithStrippedFields {
    pub public_field: i32,
    #[doc(hidden)]
    pub stripped_field: u32,
}
