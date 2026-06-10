Output docs/demo-evidence/STORY-081/AC-001-006-eval-title-constraint.gif
Output docs/demo-evidence/STORY-081/AC-001-006-eval-title-constraint.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 20
Set Shell "bash"

# ── Setup (hidden): navigate to worktree, warm cargo cache ───────────────────
Hide
Type "cd /Users/jmagady/Dev/slideforge/.worktrees/STORY-081"
Enter
Sleep 1s
Type "cargo build -p slideforge-eval --tests 2>/dev/null"
Enter
Sleep 60s
Show

# ── AC-001: bullet with bold markup → FieldValue::Inlines (not literal "**") ─
Type "# AC-001: bold bullet → FieldValue::Inlines, NOT FieldValue::Str('**...**')"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac001_bullet_bold_produces_field_value_inlines)' 2>&1 | tail -5"
Enter
Sleep 8s

# ── AC-001: all 8 inline forms → Inlines with correct variants ───────────────
Type "# AC-001: all 8 forms (bold/italic/code/link/sup/sub/del/highlight) → Inlines"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac001_all_8_inline_forms_in_body)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-001: body / caption / description / subtitle fields ───────────────────
Type "# AC-001: body, caption, description, subtitle fields all produce Inlines"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac001_body_inline_markup|ac001_caption_inline|ac001_description_inline|ac001_subtitle_inline)' 2>&1 | tail -8"
Enter
Sleep 8s

# ── AC-001: rejection test — no literal asterisks leak into output ────────────
Type "# AC-001: R1 anti-pattern closed — no literal '**' in FieldValue output"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac001_rejects_literal_asterisks_in_bullet)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-006 (success path): title with bold → E-EVL-015 warning emitted ───────
Type "# AC-006: title with bold markup → E-EVL-015 warning emitted into diagnostic sink"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac006_title_with_bold_emits_warning)' 2>&1 | tail -5"
Enter
Sleep 6s

# ── AC-006 (strip path): title with bold → strips to plain 'Bold Title' ──────
Type "# AC-006: title markup stripped → FieldValue::Str('Bold Title') for PPTX path"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac006_title_with_bold_strips_to_plain_str)' 2>&1 | tail -5"
Enter
Sleep 5s

# ── AC-006 (plain title): no false-positive warning ──────────────────────────
Type "# AC-006: plain title → zero E-EVL-015 warnings (no false positive)"
Enter
Type "cargo nextest run -p slideforge-eval -E 'test(ac006_plain_title_no_warning)' 2>&1 | tail -5"
Enter
Sleep 5s

# Hold final frame
Sleep 2s
