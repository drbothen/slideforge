//! Internal test modules for `slideforge-eval`.
//!
//! Tests live in a submodule to keep `src/` well-organized while allowing
//! access to private helpers via `crate::` imports.

pub mod section_register_routing_tests;
pub mod slide_inline_markup_eval_tests;
pub mod slide_source_span_tests;
