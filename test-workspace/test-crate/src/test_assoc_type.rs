/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use external_lib::SomeStruct;

pub struct PublicStructImplsTraitWithExtAssocType;

impl TryFrom<()> for PublicStructImplsTraitWithExtAssocType {
    // This should be an error since `SomeStruct` is exposed in the public API
    type Error = SomeStruct;

    fn try_from(_: ()) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

struct PrivateStructImplsTraitWithExtAssocType;

impl TryFrom<()> for PrivateStructImplsTraitWithExtAssocType {
    // This should be allowed because the struct is private
    type Error = SomeStruct;

    fn try_from(_: ()) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

trait PrivateTraitWithAssocType {
    type Something;

    fn something(&self);
}

pub struct PublicStructImplsPrivateTraitWithAssocType;

impl PrivateTraitWithAssocType for PublicStructImplsPrivateTraitWithAssocType {
    // This should be allowed because the trait is private
    type Something = SomeStruct;

    fn something(&self) {}
}

pub trait PublicTraitWithAssocType {
    type Something;

    fn something(&self);
}

pub struct PublicStructImplsPublicTraitWithAssocType;

impl PublicTraitWithAssocType for PublicStructImplsPublicTraitWithAssocType {
    // This should be an error since `SomeStruct` is exposed in the public API
    type Something = Result<(), SomeStruct>;

    fn something(&self) {}
}
