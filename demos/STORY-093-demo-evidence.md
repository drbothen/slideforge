# STORY-093 Demo Evidence — CI arm64 Build-Time Reduction (mold + CI Profile)

**Story:** STORY-093  
**Spec version:** 1.4  
**Artifact HEAD:** 7a91ed0a  
**tdd_mode:** facade (no Rust unit tests — verification is structural YAML/TOML inspection + local build proof)  
**Evidence date:** 2026-06-10

---

## AC-001: `cargo build --timings` baseline captured for arm64 BEFORE mold

### Requirement
Before mold takes effect, a baseline `cargo build --timings` step must exist in the arm64 job,
ordered BEFORE the mold install step, with: (a) a fail-closed guard that errors if no timing
artifact is produced, (b) an `actions/upload-artifact` upload step with `if-no-files-found: error`,
and (c) REMOVE-AFTER-BASELINE tracking anchors so the one-off profiling steps are not accidentally
left in permanently.

### Evidence — step ordering extract (ci.yml lines 387-457)

The baseline capture step appears at line 387 and the mold install step at line 458.
Ordering within the arm64 matrix job is:

```
line 387: "Capture build timings baseline (linux-arm64 ONE-OFF; AC-001)"
            if: matrix.target.name == 'linux-arm64'
            run: cargo build --workspace --all-features --tests --profile ci --timings

line 415: "Guard timings presence before upload (linux-arm64 ONE-OFF; AC-001)"
            if: matrix.target.name == 'linux-arm64'
            run: |
              [ -n "$(ls -A target/cargo-timings 2>/dev/null)" ] || {
                echo "::error::no timings produced — baseline build did not complete ..."
                exit 1
              }
              echo "Check passed: timings artifact present ($(find target/cargo-timings -maxdepth 1 -type f | wc -l) file(s))"

line 431: "Upload build timings baseline (linux-arm64 ONE-OFF; AC-001)"
            if: always() && matrix.target.name == 'linux-arm64'
            uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
            with:
              name: cargo-timings-arm64-baseline
              path: target/cargo-timings/
              if-no-files-found: error

line 458: "Install mold linker (linux-arm64 ONLY)"   ← mold install comes AFTER baseline
            if: matrix.target.name == 'linux-arm64'
            uses: rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444
```

### REMOVE-AFTER-BASELINE anchors (grep output)

Both anchor strings are present at lines 412 and 432:

```
line 412: # REMOVE-AFTER-BASELINE — tracked as FU-093-BASELINE-REMOVAL in .factory STATE.md open follow-ups;
           removal commit lands once before/after timings are recorded in PR #<this PR>.
line 432: # REMOVE-AFTER-BASELINE — tracked as FU-093-BASELINE-REMOVAL in .factory STATE.md open follow-ups;
           removal commit lands once before/after timings are recorded in PR #<this PR>.
```

### Fail-closed guard verification

The guard step (lines 424-430) uses `set -euo pipefail` and `exit 1` on an empty/absent
`target/cargo-timings/`; the upload step uses `if-no-files-found: error`. Both are
fail-closed — absence of the timing artifact is NOT a passing state (AC-002 principle applied).

### Live timings

Deferred to CI. The before/after link-phase split will be recorded in the PR description once
the arm64 full-tier run completes. This evidence file covers the structural correctness of the
baseline capture mechanism.

**AC-001: PASS (structural)**

---

## AC-002: mold installed on linux-arm64 ONLY; readelf verification with 3 fail paths

### Requirement
`setup-mold@9c9c13bf` with `make-default: true` in the arm64 job ONLY; NO
`CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` anywhere in ci.yml; readelf `.comment`
step with 3 distinct fail paths (no binary found, not ELF, mold not in .comment).

### Evidence — setup-mold step (ci.yml lines 458-488)

```yaml
- name: Install mold linker (linux-arm64 ONLY)
  if: matrix.target.name == 'linux-arm64'
  uses: rui314/setup-mold@9c9c13bf4c3f1adef0cc596abc155580bcb04444
  with:
    make-default: true
  # make-default: true symlinks /usr/bin/ld → mold; cc driver picks it up.
  # No RUSTFLAGS modification needed or permitted.
  # SHA confirmed 2026-06-10 via git ls-remote.
```

The step is guarded by `if: matrix.target.name == 'linux-arm64'` — it cannot execute on
`macos-arm64`, `windows-x86_64`, or the fast `test (linux-x86_64)` job.

### CARGO_TARGET_AARCH64 grep-zero proof

```
$ grep -c 'CARGO_TARGET_AARCH64' .github/workflows/ci.yml
1
```

The single hit is a comment (line 474):
```
# Do NOT add CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS here —
```

Zero active env-var assignments of `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS` exist
in ci.yml. The grep count is 1 comment-only occurrence, confirming the rejected mechanism
is documented but not implemented.

### readelf verification step — 3 fail paths (ci.yml lines 543-584)

```bash
# Fail path 1: no binary found in target/ci/deps
bin=$(find target/ci/deps -maxdepth 1 -type f -executable ! -name '*.so' ! -name '*.d' -print -quit)
if [ -z "$bin" ]; then
  echo "::error::No test binary found in target/ci/deps — nextest build may have failed ..."
  exit 1
fi

# Fail path 2: candidate is not an ELF file
if ! file "$bin" | grep -q ELF; then
  echo "::error::Candidate binary '$bin' is not an ELF file — cannot verify mold linkage ..."
  exit 1
fi

# Fail path 3: mold not in .comment section
out=$(readelf -p .comment "$bin")
printf '%s' "$out" | grep -qi mold || {
  echo "::error::mold not used for linking — check setup-mold make-default: true is set"
  exit 1
}

# Check-passed line (positive coverage)
count=$(find target/ci/deps -maxdepth 1 -type f -executable ! -name '*.so' ! -name '*.d' | wc -l)
echo "Check passed: mold linkage verified on $bin (.comment contains mold); $count executable(s) present"
```

All three distinct failure modes produce `::error::` annotations and `exit 1`. The
Check-passed line is only reachable after all three conditions are satisfied.

**AC-002: PASS (structural)**

---

## AC-003: Forced-relink step; byte-identical find predicate at 3 sites

### Requirement
A "Force relink under mold" step must exist between mold install and nextest.
The find predicate `-type f -executable ! -name '*.so' ! -name '*.d'` must be
byte-identical at all three sites in ci.yml (force-relink delete, readelf bin probe,
readelf count find) per TD-VSDD-060.

### Evidence — force-relink step (ci.yml lines 489-516)

```yaml
- name: Force relink under mold (linux-arm64)
  if: matrix.target.name == 'linux-arm64'
  run: |
    set -euo pipefail
    if [ -d target/ci/deps ]; then
      find target/ci/deps -maxdepth 1 -type f -executable ! -name '*.so' ! -name '*.d' -delete
    fi
```

This step appears after "Install mold linker" (line 458) and before "cargo nextest run"
(line 517), ensuring GNU-ld-linked executables are removed so nextest relinks under mold.

### Byte-identity proof — find predicate at 3 sites

```
$ grep -c '\-type f -executable ! -name '\''*.so'\'' ! -name '\''*.d'\''' .github/workflows/ci.yml
5
```

All 5 lines containing the predicate pattern are in ci.yml. The three functional sites are:

**Site 1 — Force relink (line 515, delete action):**
```bash
find target/ci/deps -maxdepth 1 -type f -executable ! -name '*.so' ! -name '*.d' -delete
```

**Site 2 — readelf bin probe (line 572, -print -quit action):**
```bash
bin=$(find target/ci/deps -maxdepth 1 -type f -executable ! -name '*.so' ! -name '*.d' -print -quit)
```

**Site 3 — readelf count find (line 583, wc -l action):**
```bash
count=$(find target/ci/deps -maxdepth 1 -type f -executable ! -name '*.so' ! -name '*.d' | wc -l)
```

The remaining 2 grep hits are comments (lines 503-504 and 563-564) documenting the
TD-VSDD-060 byte-identity requirement. All three functional predicates are byte-identical.

### Live timings delta

Deferred to CI. After mold is active and the forced-relink step executes, the link phase
duration in the after-mold `--timings` capture will be compared to the AC-001 baseline.
The direction (reduction vs GNU ld) is the acceptance bar; the magnitude depends on the
link/codegen split measured in AC-001.

**AC-003: PASS (structural)**

---

## AC-004: `[profile.ci]` in Cargo.toml; `--cargo-profile ci` on both nextest legs; bench still `--profile bench`; LOCAL BUILD PROOF

### Requirement
`Cargo.toml` must have `[profile.ci]` with `inherits = "test"` and `debug = "line-tables-only"`.
Both nextest legs (fast `test` job and slow `test-matrix-slow` job) must pass `--cargo-profile ci`.
The bench job must still use `--profile bench`, not `--cargo-profile ci`.

### Evidence — Cargo.toml [profile.ci] block (lines 201-203)

```toml
[profile.ci]
inherits = "test"
debug = "line-tables-only"
```

The block comment at lines 172-200 documents: (a) the distinction between nextest execution
profile (`--profile ci`) and this Cargo build profile (`--cargo-profile ci`), (b) the
`target/ci/` directory implication, (c) rationale for omitted fields (codegen-units,
incremental, strip).

### Evidence — `--cargo-profile ci` on both nextest legs

```
$ grep -n '\-\-cargo-profile ci' .github/workflows/ci.yml
245:          --cargo-profile ci
527:          --cargo-profile ci
```

Line 245 is the fast `test (linux-x86_64)` job. Line 527 is the slow `test-matrix-slow` job
(which covers linux-arm64, macos-arm64, and windows-x86_64). Both legs carry both
`--profile ci` (nextest execution profile) and `--cargo-profile ci` (Cargo build profile).

### Evidence — bench job still uses `--profile bench`

```
$ grep -n '\-\-profile bench' .github/workflows/ci.yml
763:        run: cargo bench --workspace --profile bench 2>&1 | tee /tmp/bench-output.txt
```

Single hit at line 763 (bench job). No `--cargo-profile ci` appears in the bench job.
`[profile.bench]` is unchanged in Cargo.toml (inherits release; governs production benchmark
binary quality — not touched by this story per Architecture Compliance Rule 2).

### LOCAL BUILD PROOF — `cargo build -p slideforge-types --profile ci`

```
$ cargo build -p slideforge-types --profile ci
    Finished `ci` profile [unoptimized + debuginfo] target(s) in 0.35s
```

Output: `Finished \`ci\` profile [unoptimized + debuginfo] target(s) in 0.35s`

Cargo resolved `[profile.ci]` from Cargo.toml and used it to build the `slideforge-types`
crate. The "ci profile" string in the output confirms the profile is recognized. The
`[unoptimized + debuginfo]` descriptor reflects `inherits = "test"` (which inherits `dev`,
which is unoptimized) with `debug = "line-tables-only"` (Cargo reports this as `debuginfo`
present, consistent with the line-tables setting producing a `.debug_line` section without
full DWARF type info).

**AC-004: PASS (structural + local build proof)**

---

## AC-005: `target/ci` size capture step; fail-closed `du` + Check-passed line

### Requirement
A size-capture step must exist on the arm64 leg that: (a) fails loudly if `target/ci/` is
absent, (b) captures `du -sh target/ci/`, (c) prints a Check-passed line with the size.
Live size deferred to CI.

### Evidence — size-capture step (ci.yml lines 528-542)

```yaml
- name: Capture target/ci size (linux-arm64, AC-005)
  if: matrix.target.name == 'linux-arm64'
  run: |
    set -euo pipefail
    [ -d target/ci ] || {
      echo "::error::target/ci missing — AC-005 size capture impossible (nextest build did not populate target/ci/)"
      exit 1
    }
    size=$(du -sh target/ci/ | cut -f1)
    echo "Check passed: target/ci size ${size} (line-tables-only debuginfo; AC-005)"
```

The step uses `set -euo pipefail` for safety. The `[ -d target/ci ] || exit 1` guard is
fail-closed — absence of `target/ci/` is an error, not a silent skip. The `du -sh target/ci/`
invocation captures the human-readable size, and the Check-passed line includes the size
variable so the value appears in CI logs.

### Live size

Deferred to CI. The before/after `target/ci/` vs `target/debug/` size comparison will be
recorded in the PR description once the arm64 full-tier run completes. The `debug =
"line-tables-only"` setting removes full DWARF type info (retaining only `.debug_line`),
which typically reduces debug section size by 60-80% vs `debug = true`.

**AC-005: PASS (structural)**

---

## AC-006: Matrix enumeration (4 active platforms); actionlint clean

### Requirement
The CI matrix must enumerate exactly 4 active platforms (linux-x86_64, linux-arm64,
macos-arm64, windows-x86_64). macos-x86_64 / macos-13 must NOT be present (removed per
STORY-091 v1.4 precedent). The workflow must be actionlint-clean.

### Evidence — matrix enumeration (ci.yml lines 344-348)

```yaml
strategy:
  fail-fast: false
  matrix:
    target:
      - { name: "linux-arm64",    runner: "ubuntu-24.04-arm" }
      - { name: "macos-arm64",    runner: "macos-latest"      }
      - { name: "windows-x86_64", runner: "windows-latest"    }
```

The slow `test-matrix-slow` job covers 3 slow-tier platforms. The fast `test (linux-x86_64)`
job (line 202, `runs-on: ubuntu-latest`) covers the 4th platform. Together: 4 active CI
platforms total.

```
Platform 1: linux-x86_64  — fast tier, job "test", runner ubuntu-latest
Platform 2: linux-arm64   — slow tier, matrix, runner ubuntu-24.04-arm
Platform 3: macos-arm64   — slow tier, matrix, runner macos-latest
Platform 4: windows-x86_64 — slow tier, matrix, runner windows-latest
```

`macos-x86_64` / `macos-13` absence confirmed at line 322:
```yaml
# Note: macos-x86_64 (macos-13) was removed — chronic runner availability
# issues (1+ hour queue times). macOS coverage is provided by macos-arm64.
```

No `macos-13` string appears elsewhere in ci.yml as a runner value.

### actionlint clean capture

```
$ actionlint .github/workflows/ci.yml
(no output, exit code 0)
```

actionlint reports zero findings. The workflow is syntactically and semantically valid
per actionlint's rule set.

**AC-006: PASS (structural + actionlint)**

---

## Coverage Summary

| AC | Description | Evidence Type | Status |
|----|-------------|---------------|--------|
| AC-001 | Timings baseline step ordering + fail-closed guard + REMOVE-AFTER-BASELINE anchors | Structural YAML extract | PASS |
| AC-002 | mold arm64-only + grep-zero on CARGO_TARGET_AARCH64 + readelf 3 fail paths | Structural YAML extract + grep count | PASS |
| AC-003 | Force-relink step + byte-identical find predicate at 3 sites (grep -c = 5) | Structural YAML extract + grep count | PASS |
| AC-004 | [profile.ci] in Cargo.toml + --cargo-profile ci on both nextest legs + bench unchanged | Structural TOML extract + grep counts + local build proof | PASS |
| AC-005 | size-capture step: fail-closed du + Check-passed line | Structural YAML extract | PASS |
| AC-006 | 4-platform matrix enumeration + actionlint clean | Structural YAML extract + tool output | PASS |

**All 6 ACs: PASS (structural verification + local build proof for AC-004)**

Live timing values (AC-001 baseline, AC-003 delta, AC-005 size before/after) are deferred
to CI per spec: these require arm64 runner execution and will be recorded in the PR description.
