warning: Module path for reexported type `InnerEnum` contains a `#[doc(hidden)]` module "hidden_module". Types declared in this module cannot be checked for external types
  --> test-type-exported-from-hidden-module/src/lib.rs:10:82
   |
10 | pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
   |                                                                                  ^-------^
   |
   = in re-export named `test_hidden_module_reexported_type::InnerEnum`

warning: Module path for reexported type `InnerStructA` contains a `#[doc(hidden)]` module "hidden_module". Types declared in this module cannot be checked for external types
  --> test-type-exported-from-hidden-module/src/lib.rs:10:25
   |
10 | pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
   |                         ^----------^
   |
   = in re-export named `test_hidden_module_reexported_type::InnerStructA`

warning: Module path for reexported type `InnerStructB` contains a `#[doc(hidden)]` module "hidden_module". Types declared in this module cannot be checked for external types
  --> test-type-exported-from-hidden-module/src/lib.rs:10:39
   |
10 | pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
   |                                       ^----------^
   |
   = in re-export named `test_hidden_module_reexported_type::InnerStructB`

warning: Module path for reexported type `InnerStructC` contains a `#[doc(hidden)]` module "hidden_module". Types declared in this module cannot be checked for external types
  --> test-type-exported-from-hidden-module/src/lib.rs:10:53
   |
10 | pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
   |                                                     ^----------^
   |
   = in re-export named `test_hidden_module_reexported_type::InnerStructC`

warning: Module path for reexported type `inner_fn` contains a `#[doc(hidden)]` module "hidden_module". Types declared in this module cannot be checked for external types
  --> test-type-exported-from-hidden-module/src/lib.rs:10:93
   |
10 | pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
   |                                                                                             ^------^
   |
   = in re-export named `test_hidden_module_reexported_type::inner_fn`

error: Unapproved external type `external_lib::SimpleNewType` referenced in public API
  --> test-type-exported-from-hidden-module/src/lib.rs:10:67
   |
10 | pub use hidden_module::{InnerStructA, InnerStructB, InnerStructC, SimpleNewType, InnerEnum, inner_fn};
   |                                                                   ^-----------^
   |
   = in re-export named `test_hidden_module_reexported_type::SimpleNewType`

1 errors, 5 warnings emitted
