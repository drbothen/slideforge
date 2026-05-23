# Agent Developer Factory Seed Bundle

This directory is a self-contained seed bundle for building a Rust + DSL PowerPoint generation project from scratch using an AI agent developer factory.

## What's in this bundle

```
agent-seed/
├── README.md                       # You are here — start here
├── PROJECT-SEED.md                 # The all-in-one project specification
├── PROJECT-README.md               # User-facing README template for the new project repo
├── DSL-GRAMMAR.ebnf                # Formal EBNF grammar for the slideforge DSL
├── scaffolding/                    # Starter files for the new project root
│   ├── README.md                   # How to apply the scaffolding
│   ├── .editorconfig
│   ├── rustfmt.toml
│   ├── clippy.toml
│   ├── rust-toolchain.toml
│   ├── Cargo.toml                  # Workspace root template
│   ├── .gitignore
│   ├── CHANGELOG.md
│   ├── LICENSE-MIT
│   ├── LICENSE-APACHE
│   └── .github/
│       └── workflows/
│           └── ci.yml              # GitHub Actions CI workflow
└── reference/                      # Reference Python implementation (current state)
    ├── build-incident-brief.py     # ~1900-line Python builder (23 slide types)
    ├── presentation-system.md      # Human-readable spec of current system
    ├── mss_metrics_leadership.py   # Real-world example deck (25 slides)
    └── _template.py                # Data module template
```

## Where to start

**Ingest order:**

1. **`PROJECT-SEED.md`** — Read this first. It's the complete project specification: mission, architecture, tech stack, DSL design, phased implementation plan, conventions, risks, and explicit Open Questions that require human input.
2. **`DSL-GRAMMAR.ebnf`** — Formal EBNF grammar plus per-slide-type schema. This is the source of truth for the DSL syntax; the chumsky parser implementation should match this.
3. **`scaffolding/README.md`** — How to apply the starter files in `scaffolding/` to the new project root during Phase 0.
4. **`reference/presentation-system.md`** — The human-readable specification of all 23 slide types in the current Python implementation. Use this to understand the visual contract.
5. **`reference/build-incident-brief.py`** — The authoritative behavior reference. Each `build_X_slide` function shows how a slide type is rendered.
6. **`reference/mss_metrics_leadership.py`** — A real-world 25-slide deck. Shows what realistic input looks like in the current Python API. The Rust + DSL version must produce visually-equivalent output.
7. **`reference/_template.py`** — Minimal data module template. Useful for understanding the metadata + slides structure.
8. **`PROJECT-README.md`** — User-facing README template for the new project repo. Customize and place at the new project root (separate from this seed README).

## What the factory should produce

A new Rust workspace (separate repository) implementing the specification in `PROJECT-SEED.md`. The reference Python code is for behavior reference only — **do not literally port it line-by-line**. The Rust implementation follows a Typst-style 4-phase pipeline (Parse → Evaluate → Layout → Export), not the Python builder's procedural style.

See `PROJECT-SEED.md` Section 6 for the phased implementation plan and Section 12 for agent factory instructions.

## Open questions

`PROJECT-SEED.md` Section 11 lists 7 Open Questions that require human input before or during implementation:

1. Final project name (placeholder is `slideforge`)
2. Indentation-significant vs. brace-delimited DSL syntax
3. Multi-file project support
4. Brand template input format
5. PDF/HTML exporter scope in v1.0
6. Python binding API style
7. Live preview architecture

The factory must pause and confirm with a human before answering these.

## Things the factory must NOT do

- Invent new slide types (the 23 are the contract)
- Redesign the visual style (brand colors and font sizes are fixed)
- Modify anything in `./reference/` (it's the source of truth)
- Pick the project name unilaterally
- Skip tests for any public API
- Carry forward Python-specific implementation quirks

## Things the factory must DO

- Read `PROJECT-SEED.md` in full before starting any phase
- Maintain visual parity with the Python tool (see Appendix A in the seed)
- Snapshot-test every slide type's rendered XML
- Use `chumsky 0.10+` for parsing and `ooxmlsdk 0.6+` for OOXML serialization
- Follow the conventions in Section 8 (code style, testing, docs)
- Update `CHANGELOG.md` with every change

## License

The seed bundle (this directory) is intended to be consumed by an AI agent factory. The resulting project should be released under MIT OR Apache-2.0 dual license (see `PROJECT-SEED.md` Section 1).

The reference Python code in `./reference/` is excerpted from a private incident response repository for use as a behavior reference. It is not licensed for redistribution; treat it as internal-use-only material that informs the new implementation.

## Manifest

| File | Purpose | Size | Read order |
|------|---------|------|-----------|
| `PROJECT-SEED.md` | Full project specification | ~35 KB | 1 |
| `DSL-GRAMMAR.ebnf` | Formal DSL grammar + slide-type schemas | ~15 KB | 2 |
| `scaffolding/README.md` | Starter files application guide | ~4 KB | 3 |
| `scaffolding/Cargo.toml` | Workspace root template | ~2 KB | Phase 0 |
| `scaffolding/.github/workflows/ci.yml` | CI workflow | ~3 KB | Phase 0 |
| `scaffolding/{.editorconfig,rustfmt.toml,clippy.toml,rust-toolchain.toml,.gitignore}` | Config files | <1 KB each | Phase 0 |
| `scaffolding/{LICENSE-MIT,LICENSE-APACHE,CHANGELOG.md}` | Project metadata | ~12 KB total | Phase 0 |
| `reference/presentation-system.md` | Slide-type catalog and design rules | ~35 KB | 4 |
| `reference/build-incident-brief.py` | Behavior reference (Python) | ~92 KB | 5 |
| `reference/mss_metrics_leadership.py` | Real-world example deck | ~42 KB | 6 |
| `reference/_template.py` | Data module template | ~6 KB | Reference |
| `PROJECT-README.md` | User-facing README template for the new repo | ~5 KB | At Phase 0 finalization |

## Versioning

- Seed version: 1.0 (2026-05-23)
- Reference Python builder version: as of 2026-05-23
- Target Rust edition: 2024
- Target `chumsky` version: 0.10+
- Target `ooxmlsdk` version: 0.6+

## Bundle integrity

The reference files in `./reference/` are exact copies (not edits) of files from the source incident_response repository as of 2026-05-23. They represent the current production state of the Python tool.

---

**Ready for agent factory ingest.**
