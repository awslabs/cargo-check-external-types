error: Unapproved external type `external_lib::SimpleNewType` referenced in public API
  --> test-crate-metadata-config/src/lib.rs:34:5
   |
34 |     pub const OTHER_CONST: SimpleNewType = SimpleNewType(5);
   |     ^------------------------------------------------------^
   |
   = in struct field of `test_crate_metadata_config::AssocConstStruct::OTHER_CONST`

1 errors, 0 warnings emitted
