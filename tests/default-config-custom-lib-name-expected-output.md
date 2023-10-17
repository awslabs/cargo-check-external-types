error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:15:1
   |
15 | pub static SOME_STRUCT: SomeStruct = SomeStruct;
   | ^----------------------------------------------^
   |
   = in static value `custom_lib::SOME_STRUCT`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:16:1
   |
16 | pub const SOME_CONST: SomeStruct = SomeStruct;
   | ^--------------------------------------------^
   |
   = in constant `custom_lib::SOME_CONST`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:21:5
   |
21 |     pub static OPTIONAL_STRUCT: Option<SomeStruct> = None;
   |     ^----------------------------------------------------^
   |
   = in generic arg of `custom_lib::some_pub_mod::OPTIONAL_STRUCT`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:22:5
   |
22 |     pub const OPTIONAL_CONST: Option<SomeStruct> = None;
   |     ^--------------------------------------------------^
   |
   = in generic arg of `custom_lib::some_pub_mod::OPTIONAL_CONST`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:26:1
   |
26 | pub type ExternalReferencingTypeAlias = SomeStruct;
   | ^-------------------------------------------------^
   |
   = in type alias of `custom_lib::ExternalReferencingTypeAlias`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:27:1
   |
27 | pub type OptionalExternalReferencingTypeAlias = Option<SomeStruct>;
   | ^-----------------------------------------------------------------^
   |
   = in generic arg of `custom_lib::OptionalExternalReferencingTypeAlias`

error: Unapproved external type `external_lib::SomeStruct` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:28:1
   |
28 | pub type ExternalReferencingRawPtr = *const SomeStruct;
   | ^-----------------------------------------------------^
   |
   = in type alias of `custom_lib::ExternalReferencingRawPtr`

error: Unapproved external type `external_lib::SimpleNewType` referenced in public API
  --> test-crate-custom-lib-name/src/lib.rs:35:5
   |
35 |     pub const OTHER_CONST: SimpleNewType = SimpleNewType(5);
   |     ^------------------------------------------------------^
   |
   = in struct field of `custom_lib::AssocConstStruct::OTHER_CONST`

8 errors, 0 warnings emitted
