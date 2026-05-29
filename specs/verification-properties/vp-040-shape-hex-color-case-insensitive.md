---
document_type: verification-property
vp_id: VP-040
title: "Shape: hex color case-insensitive — lowercase = uppercase for all 6-digit forms"
module: slideforge-layout
tool: unit
phase: P3
priority: P1
status: draft
kani_amenable: true
bc_trace: [BC-3.04.001]
anchored_to_bc: BC-3.04.001
traces_to: .factory/specs/verification-properties/VP-INDEX.md
---

# VP-040: Shape Hex Color Case-Insensitive (Lowercase = Uppercase)

## Property Statement

For any valid 6-digit hex color string `#RRGGBB` where R, G, B are hexadecimal digits
(0-9, a-f, A-F), the parser MUST produce the same `Rgb { r, g, b }` value regardless
of the case of the hex digits. Specifically:
- `"#FF6F00"` and `"#ff6f00"` MUST both parse to `Rgb { r: 255, g: 111, b: 0 }`.
- The normalized `Rgb` is stored in uppercase-independent internal representation.

Formally: `parse_hex_color(s.to_uppercase()) == parse_hex_color(s.to_lowercase())`
for any valid 6-digit hex color string `s`.

## Motivation

BC-3.04.001 Invariant 5, EC-006, and EC-007. Case-insensitive hex parsing is standard
behavior for CSS/HTML color codes. Users writing `#ff6f00` (lowercase) and `#FF6F00`
(uppercase) expect the same shape fill. The `Rgb` struct stores raw `u8` values and is
case-independent by construction; the parsing step must be case-insensitive.

## Feasibility Assessment

Kani-amenable: the `parse_hex_color` function is pure — it takes a string slice and
returns `Result<Rgb, ParseError>`. Kani can verify the case-insensitivity property
over bounded hex strings using concrete examples and symbolic reasoning.

`kani_amenable: true` — pure parsing function.

## Proof Harness Skeleton

```rust
// crates/slideforge-layout/src/proofs/hex_color.rs
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn canonical_test_vector_upper_and_lower() {
        // BC-3.04.001 EC-006 and EC-007 canonical test vectors
        let upper = parse_hex_color("#FF6F00").expect("valid upper");
        let lower = parse_hex_color("#ff6f00").expect("valid lower");
        kani::assert!(upper.r == 255 && upper.g == 111 && upper.b == 0);
        kani::assert!(lower.r == 255 && lower.g == 111 && lower.b == 0);
        kani::assert!(upper == lower);
    }

    #[kani::proof]
    fn mixed_case_produces_same_rgb() {
        let mixed = parse_hex_color("#Ff6F00").expect("valid mixed");
        kani::assert!(mixed.r == 255 && mixed.g == 111 && mixed.b == 0);
    }
}
```

## Test Coverage (before Phase 6)

```rust
// crates/slideforge-layout/src/tests/hex_color.rs
#[test]
fn test_uppercase_hex_parses_correctly() {
    let rgb = parse_hex_color("#FF6F00").unwrap();
    assert_eq!(rgb, Rgb { r: 255, g: 111, b: 0 });
}

#[test]
fn test_lowercase_hex_parses_correctly() {
    let rgb = parse_hex_color("#ff6f00").unwrap();
    assert_eq!(rgb, Rgb { r: 255, g: 111, b: 0 });
}

#[test]
fn test_short_form_rgb_rejected() {
    assert!(parse_hex_color("#F60").is_err());
}

#[test]
fn test_alpha_form_rejected() {
    assert!(parse_hex_color("#FF6F00FF").is_err());
}
```

## Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `"#FF6F00"` | `Rgb { r: 255, g: 111, b: 0 }` | happy-path (EC-006) |
| `"#ff6f00"` | `Rgb { r: 255, g: 111, b: 0 }` | case-insensitive (EC-007) |
| `"#Ff6F00"` | `Rgb { r: 255, g: 111, b: 0 }` | mixed-case |
| `"#F60"` | `ParseError::InvalidHexColor` | error (EC-008) |
| `"#FF6F00FF"` | `ParseError::InvalidHexColor` | error (EC-009) |
