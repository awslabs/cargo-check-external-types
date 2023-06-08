/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub struct LocalUnitStruct;

pub struct TupleStructWithExternalType(pub external_lib::SomeStruct);
pub struct TupleStructWithoutExternalType(pub LocalUnitStruct);
pub struct TupleStructWithPrivateExternalType(external_lib::SomeStruct);
pub struct TupleStructWithoutPrivateExternalType(LocalUnitStruct);

pub struct PlainStructWithExternalType {
    pub external: external_lib::SomeStruct,
}
pub struct PlainStructWithoutExternalType {
    pub not_external: LocalUnitStruct,
}
pub struct PlainStructWithPrivateExternalType {
    external: external_lib::SomeStruct,
}
pub struct PlainStructWithoutPrivateExternalType {
    not_external: LocalUnitStruct,
}

pub struct ImplsGenericTrait;
impl external_lib::SimpleGenericTrait<external_lib::SomeStruct> for ImplsGenericTrait {
    fn something(&self, _thing: external_lib::SomeStruct) -> u32 {
        0
    }
}
