error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
  --> test-crate/src/lib.rs:37:1
   |
37 | pub fn external_in_fn_input(_one: &SomeStruct, _two: impl SimpleTrait) {}
   | ^-----------------------------------------------------------------------^
   |
   = in argument named `_two` of `test_crate::external_in_fn_input`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
  --> test-crate/src/lib.rs:37:1
   |
37 | pub fn external_in_fn_input(_one: &SomeStruct, _two: impl SimpleTrait) {}
   | ^-----------------------------------------------------------------------^
   |
   = in trait bound of `test_crate::external_in_fn_input`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:37:1
   |
37 | pub fn external_in_fn_input(_one: &SomeStruct, _two: impl SimpleTrait) {}
   | ^-----------------------------------------------------------------------^
   |
   = in argument named `_one` of `test_crate::external_in_fn_input`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:42:1
   |
42 | pub fn external_in_fn_output() -> SomeStruct {
   | ...
44 | }␊
   | ^
   |
   = in return value of `test_crate::external_in_fn_output`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
  --> test-crate/src/lib.rs:46:1
   |
46 | pub fn external_opaque_type_in_output() -> impl SimpleTrait {
   | ...
48 | }␊
   | ^
   |
   = in return value of `test_crate::external_opaque_type_in_output`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:53:1
   |
53 | pub fn external_in_fn_output_generic() -> Option<SomeStruct> {
   | ...
55 | }␊
   | ^
   |
   = in generic arg of `test_crate::external_in_fn_output_generic`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:61:5
   |
61 |     pub fn something(_one: &SomeStruct) {}
   |     ^------------------------------------^
   |
   = in argument named `_one` of `test_crate::something`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:66:5
   |
66 |     pub field: SomeStruct,
   |     ^-------------------^
   |
   = in struct field of `test_crate::StructWithExternalFields::field`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:67:5
   |
67 |     pub optional_field: Option<SomeStruct>,
   |     ^------------------------------------^
   |
   = in generic arg of `test_crate::StructWithExternalFields::optional_field`

error: Unapproved external type `external_lib::SomeOtherStruct` referenced in public API
  --> test-crate/src/lib.rs:71:5
   |
71 |     pub fn new(_field: impl Into<SomeStruct>, _optional_field: Option<SomeOtherStruct>) -> Self {
   | ...
73 |     }␊
   |     ^
   |
   = in generic arg of `test_crate::StructWithExternalFields::new`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:71:5
   |
71 |     pub fn new(_field: impl Into<SomeStruct>, _optional_field: Option<SomeOtherStruct>) -> Self {
   | ...
73 |     }␊
   |     ^
   |
   = in generic arg of `test_crate::StructWithExternalFields::new`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:77:5
   |
77 |     fn something(&self, a: SomeStruct) -> LocalStruct;
   |     ^------------------------------------------------^
   |
   = in argument named `a` of `test_crate::TraitReferencingExternals::something`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:78:5
   |
78 |     fn optional_something(&self, a: Option<SomeStruct>) -> LocalStruct;
   |     ^-----------------------------------------------------------------^
   |
   = in generic arg of `test_crate::TraitReferencingExternals::optional_something`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:79:5
   |
79 |     fn otherthing(&self) -> SomeStruct;
   |     ^---------------------------------^
   |
   = in return value of `test_crate::TraitReferencingExternals::otherthing`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:80:5
   |
80 |     fn optional_otherthing(&self) -> Option<SomeStruct>;
   |     ^--------------------------------------------------^
   |
   = in generic arg of `test_crate::TraitReferencingExternals::optional_otherthing`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:83:1
   |
83 | pub enum EnumWithExternals<T = SomeStruct> {
   | ...
97 | }␊
   | ^
   |
   = in generic default binding of `test_crate::EnumWithExternals`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:88:15
   |
88 |     TupleEnum(SomeStruct, Box<dyn SimpleTrait>),
   |               ^--------^
   |
   = in struct field of `test_crate::EnumWithExternals::TupleEnum::0`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
  --> test-crate/src/lib.rs:88:27
   |
88 |     TupleEnum(SomeStruct, Box<dyn SimpleTrait>),
   |                           ^------------------^
   |
   = in dyn trait of `test_crate::EnumWithExternals::TupleEnum::1`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/lib.rs:90:9
   |
90 |         some_struct: SomeStruct,
   |         ^---------------------^
   |
   = in struct field of `test_crate::EnumWithExternals::StructEnum::some_struct`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
  --> test-crate/src/lib.rs:91:9
   |
91 |         simple_trait: Box<dyn SimpleTrait>,
   |         ^--------------------------------^
   |
   = in dyn trait of `test_crate::EnumWithExternals::StructEnum::simple_trait`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
   --> test-crate/src/lib.rs:103:5
    |
103 |     pub fn another_thing<S: SimpleTrait>(_s: S) -> Self {
    | ...
105 |     }␊
    |     ^
    |
    = in trait bound of `test_crate::EnumWithExternals::another_thing`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:108:1
    |
108 | pub static SOME_STRUCT: SomeStruct = SomeStruct;
    | ^----------------------------------------------^
    |
    = in static value `test_crate::SOME_STRUCT`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:109:1
    |
109 | pub const SOME_CONST: SomeStruct = SomeStruct;
    | ^--------------------------------------------^
    |
    = in constant `test_crate::SOME_CONST`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:114:5
    |
114 |     pub static OPTIONAL_STRUCT: Option<SomeStruct> = None;
    |     ^----------------------------------------------------^
    |
    = in generic arg of `test_crate::some_pub_mod::OPTIONAL_STRUCT`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:115:5
    |
115 |     pub const OPTIONAL_CONST: Option<SomeStruct> = None;
    |     ^--------------------------------------------------^
    |
    = in generic arg of `test_crate::some_pub_mod::OPTIONAL_CONST`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:119:1
    |
119 | pub type ExternalReferencingTypeAlias = SomeStruct;
    | ^-------------------------------------------------^
    |
    = in type alias of `test_crate::ExternalReferencingTypeAlias`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:120:1
    |
120 | pub type OptionalExternalReferencingTypeAlias = Option<SomeStruct>;
    | ^-----------------------------------------------------------------^
    |
    = in generic arg of `test_crate::OptionalExternalReferencingTypeAlias`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
   --> test-crate/src/lib.rs:121:1
    |
121 | pub type DynExternalReferencingTypeAlias = Box<dyn SimpleTrait>;
    | ^--------------------------------------------------------------^
    |
    = in dyn trait of `test_crate::DynExternalReferencingTypeAlias`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:122:1
    |
122 | pub type ExternalReferencingRawPtr = *const SomeStruct;
    | ^-----------------------------------------------------^
    |
    = in type alias of `test_crate::ExternalReferencingRawPtr`

error: Unapproved external type `external_lib::AssociatedGenericTrait` referenced in public API
   --> test-crate/src/lib.rs:124:1
    |
124 | pub fn fn_with_external_trait_bounds<I, O, E, T>(_thing: T)
    | ...
131 | }␊
    | ^
    |
    = in trait bound of `test_crate::fn_with_external_trait_bounds`

error: Unapproved external type `external_lib::SomeOtherStruct` referenced in public API
   --> test-crate/src/lib.rs:124:1
    |
124 | pub fn fn_with_external_trait_bounds<I, O, E, T>(_thing: T)
    | ...
131 | }␊
    | ^
    |
    = in generic arg of `test_crate::fn_with_external_trait_bounds`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:124:1
    |
124 | pub fn fn_with_external_trait_bounds<I, O, E, T>(_thing: T)
    | ...
131 | }␊
    | ^
    |
    = in generic arg of `test_crate::fn_with_external_trait_bounds`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
   --> test-crate/src/lib.rs:134:5
    |
134 |     type Thing: SimpleTrait;
    |     ^----------------------^
    |
    = in trait bound of `test_crate::SomeTraitWithExternalDefaultTypes::Thing`

error: Unapproved external type `external_lib::AssociatedGenericTrait` referenced in public API
   --> test-crate/src/lib.rs:135:5
    |
135 |     type OtherThing: AssociatedGenericTrait<
    | ...
139 |     >;␊
    |     ^^
    |
    = in trait bound of `test_crate::SomeTraitWithExternalDefaultTypes::OtherThing`

error: Unapproved external type `external_lib::SomeOtherStruct` referenced in public API
   --> test-crate/src/lib.rs:135:5
    |
135 |     type OtherThing: AssociatedGenericTrait<
    | ...
139 |     >;␊
    |     ^^
    |
    = in generic default binding of `test_crate::SomeTraitWithExternalDefaultTypes::OtherThing`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
   --> test-crate/src/lib.rs:135:5
    |
135 |     type OtherThing: AssociatedGenericTrait<
    | ...
139 |     >;␊
    |     ^^
    |
    = in generic default binding of `test_crate::SomeTraitWithExternalDefaultTypes::OtherThing`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
   --> test-crate/src/lib.rs:145:5
    |
145 |     type MyGAT<T>
    | ...
147 |         T: SimpleTrait;␊
    |     ^-----------------^
    |
    = in trait bound of `test_crate::SomeTraitWithGenericAssociatedType::MyGAT`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
   --> test-crate/src/lib.rs:149:5
    |
149 |     fn some_fn<T: SimpleTrait>(&self, thing: Self::MyGAT<T>);
    |     ^-------------------------------------------------------^
    |
    = in trait bound of `test_crate::SomeTraitWithGenericAssociatedType::some_fn`

error: Unapproved external type `external_lib::SimpleNewType` referenced in public API
   --> test-crate/src/lib.rs:157:5
    |
157 |     pub const OTHER_CONST: SimpleNewType = SimpleNewType(5);
    |     ^------------------------------------------------------^
    |
    = in struct field of `test_crate::AssocConstStruct::OTHER_CONST`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/test_assoc_type.rs:12:5
   |
12 |     type Error = SomeStruct;
   |     ^----------------------^
   |
   = in associated type `test_crate::test_assoc_type::PublicStructImplsTraitWithExtAssocType::Error`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/test_assoc_type.rs:55:5
   |
55 |     type Something = Result<(), SomeStruct>;
   |     ^--------------------------------------^
   |
   = in generic arg of `test_crate::test_assoc_type::PublicStructImplsPublicTraitWithAssocType::Something`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
 --> test-crate/src/test_structs.rs:8:40
  |
8 | pub struct TupleStructWithExternalType(pub external_lib::SomeStruct);
  |                                        ^--------------------------^
  |
  = in struct field of `test_crate::test_structs::TupleStructWithExternalType::0`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/test_structs.rs:14:5
   |
14 |     pub external: external_lib::SomeStruct,
   |     ^------------------------------------^
   |
   = in struct field of `test_crate::test_structs::PlainStructWithExternalType::external`

error: Unapproved external type `external_lib::SimpleGenericTrait` referenced in public API
  --> test-crate/src/test_structs.rs:27:1
   |
27 | impl external_lib::SimpleGenericTrait<external_lib::SomeStruct> for ImplsGenericTrait {
   | ...
31 | }␊
   | ^
   |
   = in implemented trait of `test_crate::test_structs::ImplsGenericTrait`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate/src/test_structs.rs:27:1
   |
27 | impl external_lib::SimpleGenericTrait<external_lib::SomeStruct> for ImplsGenericTrait {
   | ...
31 | }␊
   | ^
   |
   = in generic arg of `test_crate::test_structs::ImplsGenericTrait`

error: Unapproved external type `external_lib::ReprCType` referenced in public API
  --> test-crate/src/test_union.rs:10:5
   |
10 |     pub repr_c: ReprCType,
   |     ^-------------------^
   |
   = in struct field of `test_crate::test_union::SimpleUnion::repr_c`

error: Unapproved external type `external_lib::ReprCType` referenced in public API
  --> test-crate/src/test_union.rs:15:5
   |
15 |     pub fn repr_c(&self) -> &ReprCType {
   | ...
17 |     }␊
   |     ^
   |
   = in return value of `test_crate::test_union::SimpleUnion::repr_c`

error: Unapproved external type `external_lib::SimpleTrait` referenced in public API
  --> test-crate/src/test_union.rs:21:1
   |
21 | pub union GenericUnion<T: Copy + SimpleTrait> {
   | ...
24 | }␊
   | ^
   |
   = in trait bound of `test_crate::test_union::GenericUnion`

warning: Fields on `test_crate::test_fields_stripped::SomeStructWithStrippedFields` marked `#[doc(hidden)]` cannot be checked for external types
48 errors, 1 warnings emitted
