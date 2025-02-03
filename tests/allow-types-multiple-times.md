warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
  --> test-crate/src/lib.rs:37:1
   |
37 | pub fn external_in_fn_input(_one: &SomeStruct, _two: impl SimpleTrait) {}
   | ^-----------------------------------------------------------------------^
   |
   = in argument named `_two` of `test_crate::external_in_fn_input`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
  --> test-crate/src/lib.rs:37:1
   |
37 | pub fn external_in_fn_input(_one: &SomeStruct, _two: impl SimpleTrait) {}
   | ^-----------------------------------------------------------------------^
   |
   = in trait bound of `test_crate::external_in_fn_input`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
  --> test-crate/src/lib.rs:46:1
   |
46 | pub fn external_opaque_type_in_output() -> impl SimpleTrait {
   | ...
48 | }␊
   | ^
   |
   = in return value of `test_crate::external_opaque_type_in_output`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
  --> test-crate/src/lib.rs:88:27
   |
88 |     TupleEnum(SomeStruct, Box<dyn SimpleTrait>),
   |                           ^------------------^
   |
   = in dyn trait of `test_crate::EnumWithExternals::TupleEnum::1`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
  --> test-crate/src/lib.rs:91:9
   |
91 |         simple_trait: Box<dyn SimpleTrait>,
   |         ^--------------------------------^
   |
   = in dyn trait of `test_crate::EnumWithExternals::StructEnum::simple_trait`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
   --> test-crate/src/lib.rs:103:5
    |
103 |     pub fn another_thing<S: SimpleTrait>(_s: S) -> Self {
    | ...
105 |     }␊
    |     ^
    |
    = in trait bound of `test_crate::EnumWithExternals::another_thing`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
   --> test-crate/src/lib.rs:121:1
    |
121 | pub type DynExternalReferencingTypeAlias = Box<dyn SimpleTrait>;
    | ^--------------------------------------------------------------^
    |
    = in dyn trait of `test_crate::DynExternalReferencingTypeAlias`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
   --> test-crate/src/lib.rs:134:5
    |
134 |     type Thing: SimpleTrait;
    |     ^----------------------^
    |
    = in trait bound of `test_crate::SomeTraitWithExternalDefaultTypes::Thing`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
   --> test-crate/src/lib.rs:145:5
    |
145 |     type MyGAT<T>
    | ...
147 |         T: SimpleTrait;␊
    |     ^-----------------^
    |
    = in trait bound of `test_crate::SomeTraitWithGenericAssociatedType::MyGAT`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
   --> test-crate/src/lib.rs:149:5
    |
149 |     fn some_fn<T: SimpleTrait>(&self, thing: Self::MyGAT<T>);
    |     ^-------------------------------------------------------^
    |
    = in trait bound of `test_crate::SomeTraitWithGenericAssociatedType::some_fn`

warning: argument named `arg0` of test_crate::hidden_arg references a hidden item. Items marked `#[doc(hidden)]` cannot be checked for external types
   --> test-crate/src/lib.rs:160:1
    |
160 | pub fn hidden_arg(arg0: HiddenStruct) {
    | ...
162 | }␊
    | ^
    |
    = in argument named `arg0` of `test_crate::hidden_arg`

warning: External type `external_lib::SimpleTrait` is allowed multiple times:
 Allowed patterns:
    - external_lib::*
    - external_lib::SimpleTrait
  --> test-crate/src/test_union.rs:21:1
   |
21 | pub union GenericUnion<T: Copy + SimpleTrait> {
   | ...
24 | }␊
   | ^
   |
   = in trait bound of `test_crate::test_union::GenericUnion`

warning: Fields on `test_crate::test_fields_stripped::SomeStructWithStrippedFields` marked `#[doc(hidden)]` cannot be checked for external types
0 errors, 13 warnings emitted
