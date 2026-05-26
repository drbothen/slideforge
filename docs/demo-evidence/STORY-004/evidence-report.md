# Demo Evidence Report — STORY-004: Value System + EMU Types

**Story:** STORY-004 — Value System + EMU Types
**Date:** 2026-05-25
**Evidence type:** Compilation + test suite (no interactive demos — pure type crate)
**Branch:** feature/S-1.04

## Rationale: No VHS/Playwright Recordings Required

STORY-004 expands the `slideforge-types` crate with the full Value type system and EMU
arithmetic. All 14 ACs are verified by one or more of:

1. **Compilation** — the crate compiles cleanly (`cargo build -p slideforge-types`)
2. **compile_fail doctests** — 4 doctests verify that coercion impls do NOT exist at compiler level
3. **Unit test suite** — 189 unit tests all pass
4. **Doc tests** — 19 doc-example tests pass
5. **Clippy** — `cargo clippy -p slideforge-types -- -D warnings` zero warnings
6. **Doc build** — `RUSTDOCFLAGS="-D warnings" cargo doc -p slideforge-types --no-deps` passes

There are no interactive user-facing features (no CLI commands, no rendered output, no UI) in this
crate. All correctness guarantees are expressed as type-level constraints enforced at compile time
and as Rust tests. The most critical guarantee (no-coercion invariant BC-1.02.003) is enforced by
the Rust compiler itself — 4 `compile_fail` doctests prove this.

## Test Suite Output (2026-05-25)

```
running 189 tests
test error::tests::test_bc_1_01_type_error_clone ... ok
test error::tests::test_bc_1_01_type_error_debug ... ok
... [185 more unit tests — all ok]
test value::tests::test_bc_1_02_003_str_no_stays_str ... ok
test value::tests::test_bc_1_02_003_type_kind_all_variants ... ok
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Doc-tests slideforge_types

running 19 tests
test crates/slideforge-types/src/emu.rs - emu::Emu::from_inches (line 59) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::from_points (line 74) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::to_points (line 116) ... ok
test crates/slideforge-types/src/emu.rs - emu::Emu::to_inches (line 102) ... ok
test crates/slideforge-types/src/value.rs - value::Value::is_truthy (line 241) ... ok
test crates/slideforge-types/src/value.rs - value::Value::type_kind (line 197) ... ok
test crates/slideforge-types/src/value.rs - value::Value::as_str (line 108) ... ok
... [12 more doc tests — all ok]
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 4 tests
test crates/slideforge-types/src/value.rs - value::Value (line 28) - compile fail ... ok
test crates/slideforge-types/src/value.rs - value::Value (line 33) - compile fail ... ok
test crates/slideforge-types/src/value.rs - value::Value (line 38) - compile fail ... ok
test crates/slideforge-types/src/value.rs - value::Value (line 43) - compile fail ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

TOTAL: 212/212 tests pass
```

## Per-AC Evidence

| AC | Verification Method | Evidence Location | Status |
|----|--------------------|--------------------|--------|
| AC-001 | 4 `compile_fail` doctests: `From<bool>`, `From<i64>`, `From<f64>`, `From<&str>` do not exist | `src/value.rs` lines 28, 33, 38, 43 | PASS |
| AC-002 | `test_bc_1_02_003_str_no_stays_str`: `Value::Str("NO") != Value::Bool(false)` | `src/value.rs` tests | PASS |
| AC-003 | `test_bc_1_02_003_float_precision_preserved`: `as_str().unwrap() == "1.10"` exact | `src/value.rs` tests | PASS |
| AC-004 | `TypeKind` enum with 7 variants; Display impl produces "string", "integer", "float", "boolean", "list", "map", "null" | `src/type_kind.rs` | PASS |
| AC-005 | `test_bc_1_02_003_type_kind_all_variants`: all 7 variant->TypeKind mappings verified | `src/value.rs` tests | PASS |
| AC-006 | `test_bc_1_02_003_is_truthy_*`: 10 tests covering Bool, Int, Float, Str, List, Map, Null, NaN | `src/value.rs` tests | PASS |
| AC-007 | `MergePrecedence` 11-variant enum; `Ord` ordering verified by `precedence_ordering` test | `src/precedence.rs` tests | PASS |
| AC-008 | Doctest: `Emu::from_inches(1.0) == Emu(914_400)` | `src/emu.rs` line 59 | PASS |
| AC-009 | Doctest: `Emu::from_points(1.0) == Emu(12_700)` | `src/emu.rs` line 74 | PASS |
| AC-010 | `emu_arithmetic` test: Add/Sub/Mul/Div/Neg; `slide_dimensions` test: 10in=9_144_000 EMU | `src/emu.rs` tests | PASS |
| AC-011 | Doctests: `to_points` roundtrip; `to_inches` roundtrip | `src/emu.rs` lines 102, 116 | PASS |
| AC-012 | `test_bc_1_02_003_as_str_does_not_coerce_int`: `Value::Int(1).as_str() == None` | `src/value.rs` tests | PASS |
| AC-013 | `cargo clippy -p slideforge-types -- -D warnings` exits 0 | CI clippy gate | PASS |
| AC-014 | `RUSTDOCFLAGS="-D warnings" cargo doc -p slideforge-types --no-deps` exits 0 | Doc build gate | PASS |
