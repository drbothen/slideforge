# slideforge-math

Math rendering engine for slideforge: LaTeX parser, `@{var}` interpolation,
OMML/MathML output, and PDF/SVG vector paths via embedded Latin Modern Math
glyph outlines.

## Features

- LaTeX subset parser (recursive-descent, error-accumulating)
- `@{var}` interpolation in math expressions
- Output formats: OMML (PPTX), MathML (HTML), PDF vector paths (SVG)
- Pure-Rust — no system font installation required
- All `<text>` nodes converted to `<path d="...">` for PDF/UA-1 compliance

## Bundled Font — Latin Modern Math

This crate embeds **Latin Modern Math** (version 1.959, ~733 KB OTF) to
enable pure-Rust PDF glyph outline rendering without requiring a system font.

**License:** GUST Font License (GFL) v1.0, compatible with LPPL 1.3c or later.

**Copyright:** Copyright 2012--2014 for Latin Modern Math OTF by B. Jackowski,
P. Strzelczyk and P. Pianowski (on behalf of TeX Users Groups).

**Source:** <https://www.gust.org.pl/projects/e-foundry/lm-math>

The complete license text, FONTLOG, and asset integrity manifest (SHA-256) are
in the [`fonts/`](fonts/) directory:

- [`fonts/LICENSE-LatinModernMath.txt`](fonts/LICENSE-LatinModernMath.txt) — GFL v1.0 + FONTLOG
- [`fonts/MANIFEST.toml`](fonts/MANIFEST.toml) — asset audit trail (SHA-256, version, source URL)
