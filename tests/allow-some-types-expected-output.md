error: Unapproved external type `external_lib::AssociatedGenericTrait` referenced in public API
   --> test-crate/src/lib.rs:124:1
    |
124 | pub fn fn_with_external_trait_bounds<I, O, E, T>(_thing: T)
    | ...
131 | }␊
    | ^
    |
    = in trait bound of `test_crate::fn_with_external_trait_bounds`

error: Unapproved external type `external_lib::AssociatedGenericTrait` referenced in public API
   --> test-crate/src/lib.rs:135:5
    |
135 |     type OtherThing: AssociatedGenericTrait<
    | ...
139 |     >;␊
    |     ^^
    |
    = in trait bound of `test_crate::SomeTraitWithExternalDefaultTypes::OtherThing`

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

warning: Fields on `test_crate::test_fields_stripped::SomeStructWithStrippedFields` marked `#[doc(hidden)]` cannot be checked for external types
4 errors, 1 warnings emitted
