warning: Approved external type `external_lib::T*` wasn't referenced in public API
warning: argument named `arg0` of test_crate::hidden_arg references a hidden item. Items marked `#[doc(hidden)]` cannot be checked for external types
   --> test-crate/src/lib.rs:160:1
    |
160 | pub fn hidden_arg(arg0: HiddenStruct) {
    | ...
162 | }␊
    | ^
    |
    = in argument named `arg0` of `test_crate::hidden_arg`

warning: Fields on `test_crate::test_fields_stripped::SomeStructWithStrippedFields` marked `#[doc(hidden)]` cannot be checked for external types
0 errors, 3 warnings emitted
