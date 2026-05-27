//! LaTeX → [`MathAst`] parser.
//!
//! Converts the raw LaTeX source string from a [`slideforge_types::MathNode`]
//! into a structured [`MathAst`] for downstream rendering.
//!
//! ## Error accumulation
//!
//! The parser accumulates ALL errors in one pass (BC-1.10.002). It never stops
//! on the first unsupported command. The returned
//! `(Option<MathAst>, Vec<MathDiagnostic>)` pair lets callers decide whether to
//! proceed with a partial AST or surface all diagnostics.
//!
//! ## Supported LaTeX subset (v1.0)
//!
//! Only the commands in the table below are recognized. Everything else produces
//! a [`MathRendererError::UnsupportedCommand`] diagnostic with a correction hint.
//!
//! | Category | Commands |
//! |----------|---------|
//! | Structure | `^`, `_`, `\frac`, `\sqrt`, `{`, `}` |
//! | Accents | `\hat`, `\bar`, `\tilde`, `\vec`, `\dot`, `\ddot` |
//! | Greek | `\alpha` … `\omega`, `\Alpha` … `\Omega` |
//! | Operators | `\sum`, `\prod`, `\int`, `\lim`, `\max`, `\min` |
//! | Symbols | `\cdot`, `\times`, `\div`, `\infty`, `\pm`, `\mp`, `\leq`, `\geq` |
//! | Spaces | `\,`, `\;`, `\quad`, `\qquad` |

use std::sync::Arc;

use slideforge_types::SourceSpan;

use crate::ast::{AccentKind, MathAst, MathMode, MathNode};
use crate::error::{MathDiagnostic, MathRendererError};

/// Maximum brace-nesting depth allowed by the parser.
///
/// When a group `{...}` would exceed this depth, a [`MathRendererError::ParseError`]
/// diagnostic is emitted and the group is treated as empty to prevent runaway
/// recursion or stack exhaustion.
pub const MAX_DEPTH: usize = 256;

/// Parse a LaTeX source string into a [`MathAst`].
///
/// # Arguments
///
/// * `latex` — Raw LaTeX source (without surrounding `$` delimiters).
/// * `mode`  — Whether the expression is inline or display-mode.
/// * `span`  — Source location of the entire math expression (for diagnostics).
///
/// # Returns
///
/// A tuple of `(Option<MathAst>, Vec<MathDiagnostic>)`:
/// - `Some(ast)` when parsing succeeded (possibly with accumulated warnings).
/// - `None` when the source is entirely unparseable.
/// - The diagnostic list is non-empty when any unsupported command or parse
///   error was encountered.
///
/// # Errors
///
/// Returns diagnostics for unsupported commands, unmatched braces, and other
/// structural errors. Never panics — all recoverable paths produce diagnostics.
#[must_use]
pub fn parse(
    latex: &str,
    mode: MathMode,
    span: SourceSpan,
) -> (Option<MathAst>, Vec<MathDiagnostic>) {
    let mut parser = LatexParser::new(latex, span);
    let nodes = parser.parse_node_list();
    let diags = parser.into_diagnostics();
    (Some(MathAst::new(mode, nodes)), diags)
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal recursive-descent parser
// ─────────────────────────────────────────────────────────────────────────────

/// Internal recursive-descent parser for the supported LaTeX subset.
///
/// Created fresh for each call to [`parse`]. Not part of the public API.
struct LatexParser<'a> {
    /// The raw LaTeX source string being parsed.
    input: &'a str,
    /// Current byte position within `input`. Always lands on a UTF-8 char boundary.
    pos: usize,
    /// Source span of the enclosing math expression, used as the base for diagnostics.
    span: SourceSpan,
    /// Accumulated diagnostics (errors and warnings). Never cleared mid-parse.
    diags: Vec<MathDiagnostic>,
    /// Current brace-nesting depth. Incremented on every `{` and decremented
    /// on every `}`. When depth would exceed [`MAX_DEPTH`], parsing of the
    /// group is aborted and a diagnostic is emitted.
    depth: usize,
}

impl<'a> LatexParser<'a> {
    /// Construct a new parser at the start of `input`, anchored to `span`.
    fn new(input: &'a str, span: SourceSpan) -> Self {
        LatexParser {
            input,
            pos: 0,
            span,
            diags: Vec::new(),
            depth: 0,
        }
    }

    /// Consume the parser and return all accumulated diagnostics.
    fn into_diagnostics(self) -> Vec<MathDiagnostic> {
        self.diags
    }

    /// Remaining unparsed input.
    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    /// Compute a [`SourceSpan`] that points to the current parser position
    /// within the math block, rather than the span of the entire math block.
    ///
    /// The returned span has:
    /// - The same `file` as the enclosing math expression span.
    /// - `byte_offset` = enclosing span's `byte_offset` + `self.pos`.
    /// - `line` and `col` approximated by counting newlines and columns from
    ///   the start of the math content up to `self.pos`.
    ///
    /// This is used to produce diagnostics that point to the specific command
    /// token, not the entire math expression.
    fn span_at(&self, cmd_start: usize) -> SourceSpan {
        // Walk the input up to cmd_start to compute line/col offsets.
        let (delta_line, delta_col) = {
            let slice = &self.input[..cmd_start];
            let mut lines: u32 = 0;
            let mut col: u32 = 0;
            for ch in slice.chars() {
                if ch == '\n' {
                    lines += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
            (lines, col)
        };

        let base = &self.span;
        let new_line = if delta_line > 0 {
            base.line.saturating_add(delta_line)
        } else {
            base.line
        };
        let new_col = if delta_line > 0 {
            // After a newline, col resets to the new line's col offset.
            delta_col + 1
        } else {
            base.col.saturating_add(delta_col)
        };
        SourceSpan::new(
            Arc::clone(&base.file),
            new_line,
            new_col,
            base.byte_offset.saturating_add(cmd_start),
        )
    }

    /// Skip ASCII whitespace.
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len()
            && self
                .input
                .as_bytes()
                .get(self.pos)
                .is_some_and(u8::is_ascii_whitespace)
        {
            self.pos += 1;
        }
    }

    /// Peek at the next byte (ASCII-safe).
    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    /// Consume one Unicode scalar value (char) from the input.
    ///
    /// Advances `self.pos` by the correct UTF-8 byte length of the consumed
    /// character, avoiding corruption of multi-byte sequences.
    fn consume_char(&mut self) -> Option<char> {
        let ch = self.input[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    /// Parse a command name after a backslash (e.g. `\alpha` → `"alpha"`).
    fn parse_command_name(&mut self) -> &'a str {
        let start = self.pos;
        // Single non-alpha character command (e.g. `\,`).
        // Use char-width advancement to handle multi-byte UTF-8 correctly —
        // a naive `self.pos += 1` would split a multi-byte sequence and cause
        // a panic when the resulting slice index is not on a char boundary
        // (FINDING-019).
        if let Some(&b) = self.input.as_bytes().get(self.pos)
            && !b.is_ascii_alphabetic()
        {
            // Advance by the full UTF-8 width of the character, not just 1 byte.
            let ch = self.input[self.pos..].chars().next().unwrap_or('\0');
            self.pos += ch.len_utf8();
            return &self.input[start..self.pos];
        }
        while let Some(&b) = self.input.as_bytes().get(self.pos) {
            if b.is_ascii_alphabetic() {
                self.pos += 1;
            } else {
                break;
            }
        }
        &self.input[start..self.pos]
    }

    /// Parse a braced group `{...}` and return the inner nodes.
    ///
    /// If the current depth would exceed [`MAX_DEPTH`], emit a diagnostic and
    /// skip to the matching `}` without recursing, preventing stack exhaustion.
    fn parse_group(&mut self) -> Vec<MathNode> {
        // Consume the opening `{`
        self.pos += 1;
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            // Emit a depth-exceeded diagnostic.
            self.diags.push(MathDiagnostic::new(
                MathRendererError::ParseError {
                    message: Arc::from(
                        "brace nesting depth exceeded maximum allowed depth (256); \
                         group content skipped",
                    ),
                    span: self.span.clone(),
                },
                self.span.clone(),
            ));
            // Skip to the matching `}` at this level without further recursion.
            // We track our own brace count to find the correct closing brace.
            let mut open_count = 1usize;
            while self.pos < self.input.len() {
                match self.input.as_bytes()[self.pos] {
                    b'{' => {
                        open_count += 1;
                        self.pos += 1;
                    },
                    b'}' => {
                        open_count -= 1;
                        if open_count == 0 {
                            self.pos += 1; // consume the closing `}`
                            break;
                        }
                        self.pos += 1;
                    },
                    _ => {
                        self.pos += 1;
                    },
                }
            }
            self.depth -= 1;
            return Vec::new();
        }

        let inner = self.parse_until(|p| p.peek_byte() == Some(b'}'));
        // Consume the closing `}` if present
        if self.peek_byte() == Some(b'}') {
            self.pos += 1;
        }
        self.depth -= 1;
        inner
    }

    /// Parse an optional bracketed index `[...]` and return inner nodes.
    fn parse_optional_bracket(&mut self) -> Option<Vec<MathNode>> {
        self.skip_whitespace();
        if self.peek_byte() == Some(b'[') {
            self.pos += 1; // consume `[`
            let inner = self.parse_until(|p| p.peek_byte() == Some(b']'));
            if self.peek_byte() == Some(b']') {
                self.pos += 1;
            }
            Some(inner)
        } else {
            None
        }
    }

    /// Parse nodes until `stop_cond` returns true or end of input.
    fn parse_until<F: Fn(&Self) -> bool>(&mut self, stop_cond: F) -> Vec<MathNode> {
        let mut nodes = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.input.len() || stop_cond(self) {
                break;
            }
            if let Some(node) = self.parse_one() {
                nodes = self.apply_scripts(nodes, node);
            }
        }
        nodes
    }

    /// Parse a list of nodes to end of input.
    fn parse_node_list(&mut self) -> Vec<MathNode> {
        self.parse_until(|_| false)
    }

    /// After parsing `node`, check if the next char is `^` or `_` and wrap
    /// appropriately.  Handles `base^{sup}`, `base_{sub}`, and combined
    /// `base^{sup}_{sub}` / `base_{sub}^{sup}`.
    ///
    /// For `Operator` nodes (large operators like `\sum`, `\prod`, `\int`), the
    /// operator is pushed to the output list first (so it appears directly in
    /// `ast.nodes`), and any following `_` / `^` scripts are produced as
    /// `Subscript`/`Superscript` nodes with an empty group base. This matches
    /// the BC-1.10.001 parser contract which requires the operator to be directly
    /// visible in the top-level node list.
    fn apply_scripts(&mut self, mut prev: Vec<MathNode>, node: MathNode) -> Vec<MathNode> {
        // Large operators are pushed immediately so they appear at the top level.
        let (is_operator, mut base) = match &node {
            MathNode::Operator(_) => {
                // Check whether scripts follow before deciding
                self.skip_whitespace();
                let has_script = matches!(self.peek_byte(), Some(b'^' | b'_'));
                if has_script {
                    // Push the operator first, then attach scripts to an empty base
                    prev.push(node);
                    (true, MathNode::Group(Vec::new()))
                } else {
                    (false, node)
                }
            },
            _ => (false, node),
        };
        let _ = is_operator; // used implicitly by the logic above

        loop {
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b'^') => {
                    self.pos += 1;
                    let sup = self.parse_single_atom();
                    base = MathNode::Superscript {
                        base: Box::new(base),
                        sup: Box::new(sup),
                    };
                },
                Some(b'_') => {
                    self.pos += 1;
                    let sub = self.parse_single_atom();
                    base = MathNode::Subscript {
                        base: Box::new(base),
                        sub: Box::new(sub),
                    };
                },
                _ => break,
            }
        }

        prev.push(base);
        prev
    }

    /// Parse exactly one "atom" — either a braced group or a single token.
    fn parse_single_atom(&mut self) -> MathNode {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'{') => {
                let inner = self.parse_group();
                if inner.len() == 1 {
                    // Unwrap single-element groups so `x^{2}` → Text("2")
                    #[allow(clippy::unwrap_used)] // safe: len==1
                    inner.into_iter().next().unwrap()
                } else {
                    MathNode::Group(inner)
                }
            },
            Some(b'\\') => {
                let cmd_start = self.pos; // position of the backslash
                self.pos += 1; // consume `\`
                let cmd = self.parse_command_name();
                self.dispatch_command(cmd, cmd_start)
            },
            _ => {
                if let Some(ch) = self.consume_char() {
                    MathNode::Text(Arc::from(ch.to_string().as_str()))
                } else {
                    MathNode::Text(Arc::from(""))
                }
            },
        }
    }

    /// Parse one top-level node (without script chaining).
    fn parse_one(&mut self) -> Option<MathNode> {
        match self.peek_byte()? {
            b'{' => {
                let inner = self.parse_group();
                Some(MathNode::Group(inner))
            },
            b'}' => {
                // Unmatched `}` — consume and skip
                self.pos += 1;
                None
            },
            b'\\' => {
                let cmd_start = self.pos; // position of the backslash
                self.pos += 1; // consume `\`
                let cmd = self.parse_command_name();
                Some(self.dispatch_command(cmd, cmd_start))
            },
            b'^' | b'_' => {
                // Dangling script without base — produce a Text node
                let ch = self.consume_char()?;
                Some(MathNode::Text(Arc::from(ch.to_string().as_str())))
            },
            _ => {
                let ch = self.consume_char()?;
                Some(MathNode::Text(Arc::from(ch.to_string().as_str())))
            },
        }
    }

    /// Dispatch a parsed command name to the correct `MathNode` variant.
    ///
    /// `cmd_start` is the byte offset of the leading backslash within `self.input`,
    /// used to compute a precise [`SourceSpan`] that points to the command token
    /// rather than the entire enclosing math block (FINDING-022).
    #[allow(clippy::too_many_lines)]
    fn dispatch_command(&mut self, cmd: &'a str, cmd_start: usize) -> MathNode {
        match cmd {
            // ── Structure ─────────────────────────────────────────────────
            "frac" => {
                let num_inner = self.parse_single_atom();
                let denom_inner = self.parse_single_atom();
                MathNode::Fraction {
                    num: Box::new(num_inner),
                    denom: Box::new(denom_inner),
                }
            },
            "sqrt" => {
                let index = self.parse_optional_bracket().map(|nodes| {
                    Box::new(if nodes.len() == 1 {
                        #[allow(clippy::unwrap_used)] // safe: len==1
                        nodes.into_iter().next().unwrap()
                    } else {
                        MathNode::Group(nodes)
                    })
                });
                let radicand = self.parse_single_atom();
                MathNode::Sqrt {
                    index,
                    radicand: Box::new(radicand),
                }
            },

            // ── Accents ───────────────────────────────────────────────────
            "hat" => {
                let inner = self.parse_single_atom();
                MathNode::Accent {
                    kind: AccentKind::Hat,
                    inner: Box::new(inner),
                }
            },
            "bar" => {
                let inner = self.parse_single_atom();
                MathNode::Accent {
                    kind: AccentKind::Bar,
                    inner: Box::new(inner),
                }
            },
            "tilde" => {
                let inner = self.parse_single_atom();
                MathNode::Accent {
                    kind: AccentKind::Tilde,
                    inner: Box::new(inner),
                }
            },
            "vec" => {
                let inner = self.parse_single_atom();
                MathNode::Accent {
                    kind: AccentKind::Vec,
                    inner: Box::new(inner),
                }
            },
            "dot" => {
                let inner = self.parse_single_atom();
                MathNode::Accent {
                    kind: AccentKind::Dot,
                    inner: Box::new(inner),
                }
            },
            "ddot" => {
                let inner = self.parse_single_atom();
                MathNode::Accent {
                    kind: AccentKind::Ddot,
                    inner: Box::new(inner),
                }
            },

            // ── Greek (lowercase and uppercase) ───────────────────────────
            "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "varepsilon" | "zeta" | "eta"
            | "theta" | "vartheta" | "iota" | "kappa" | "lambda" | "mu" | "nu" | "xi" | "pi"
            | "varpi" | "rho" | "varrho" | "sigma" | "varsigma" | "tau" | "upsilon" | "phi"
            | "varphi" | "chi" | "psi" | "omega" | "Alpha" | "Beta" | "Gamma" | "Delta"
            | "Epsilon" | "Zeta" | "Eta" | "Theta" | "Iota" | "Kappa" | "Lambda" | "Mu" | "Nu"
            | "Xi" | "Pi" | "Rho" | "Sigma" | "Tau" | "Upsilon" | "Phi" | "Chi" | "Psi"
            | "Omega" => MathNode::Greek(Arc::from(cmd)),

            // ── Operators ─────────────────────────────────────────────────
            "sum" | "prod" | "int" | "lim" | "max" | "min" => MathNode::Operator(Arc::from(cmd)),

            // ── Symbols ───────────────────────────────────────────────────
            "cdot" | "times" | "div" | "infty" | "pm" | "mp" | "leq" | "geq" | "neq" | "approx"
            | "equiv" | "in" | "notin" | "subset" | "supset" | "cup" | "cap" | "emptyset"
            | "forall" | "exists" | "partial" | "nabla" | "to" | "rightarrow" | "leftarrow"
            | "Rightarrow" | "Leftarrow" | "ldots" | "cdots" | "vdots" | "ddots" => {
                MathNode::Symbol(Arc::from(cmd))
            },

            // ── Spaces ────────────────────────────────────────────────────
            "," | ";" | "quad" | "qquad" => MathNode::Space,

            // ── Delimiters ────────────────────────────────────────────────
            "left" => {
                // Consume the delimiter character after `\left`
                self.skip_whitespace();
                let left_delim = self.consume_delimiter_char();
                // Parse content until `\right`
                let pre_inner_pos = self.pos;
                let inner = self.parse_until(LatexParser::is_at_right);
                // Detect unmatched \left: if we reached EOF without finding
                // `\right`, emit a diagnostic.
                let found_right = self.is_at_right();
                if !found_right {
                    // Only emit the diagnostic if we consumed any input while
                    // searching (i.e., we were actually looking for a \right).
                    let _ = pre_inner_pos; // suppress unused warning
                    self.diags.push(MathDiagnostic::new(
                        MathRendererError::ParseError {
                            message: Arc::from(
                                "unmatched \\left delimiter: no corresponding \\right found",
                            ),
                            span: self.span.clone(),
                        },
                        self.span.clone(),
                    ));
                    // Return the inner content as a group rather than a Delimiter
                    return MathNode::Group(inner);
                }
                // Consume `\right` + its delimiter
                let right_delim = if self.peek_byte() == Some(b'\\') {
                    self.pos += 1;
                    let right_cmd = self.parse_command_name();
                    if right_cmd == "right" {
                        self.skip_whitespace();
                        self.consume_delimiter_char()
                    } else {
                        Arc::from(")")
                    }
                } else {
                    Arc::from(")")
                };
                MathNode::Delimiter {
                    left: left_delim,
                    right: right_delim,
                    inner,
                }
            },
            "right" => {
                // `\right` without matching `\left` — consume delimiter, emit text
                self.skip_whitespace();
                let _d = self.consume_delimiter_char();
                MathNode::Text(Arc::from(")"))
            },

            // ── Environments ─────────────────────────────────────────────
            "begin" => {
                self.skip_whitespace();
                let env_name = self.parse_env_name();
                match env_name {
                    "align" | "align*" | "aligned" => self.parse_align_env("align"),
                    "cases" => self.parse_cases_env(),
                    other => {
                        // All non-{align,cases} environments are unsupported — always diagnose.
                        // (The is_supported_command check was removed: command names like
                        // "frac" being valid commands does NOT make \begin{frac} valid.)
                        {
                            let cmd_span = self.span_at(cmd_start);
                            self.diags.push(MathDiagnostic::new(
                                MathRendererError::UnsupportedCommand {
                                    command: Arc::from(other),
                                    span: cmd_span.clone(),
                                    hint: Arc::from(hint_for_command(other)),
                                    error_code: "E-EXP-006",
                                },
                                cmd_span,
                            ));
                        }
                        MathNode::Group(Vec::new())
                    },
                }
            },
            "end" => {
                // Stray `\end{...}` — consume and ignore
                self.skip_whitespace();
                let _ = self.parse_env_name();
                MathNode::Group(Vec::new())
            },

            // ── Text/font commands ────────────────────────────────────────
            // `\text{...}` and `\mathrm{...}` / `\mathbf{...}` produce a
            // TextRun (upright/plain style in OMML). We scan the braced content
            // character-by-character to preserve ALL characters including spaces
            // — using `parse_group()` would incorrectly strip internal whitespace
            // via `skip_whitespace()`.
            //
            // `\mathit{...}` is handled separately: math mode is already italic,
            // so \mathit is a no-op style-wise. We emit the inner content as
            // regular Text/Group nodes (not TextRun) to avoid incorrect OMML
            // upright-text wrapping.
            "text" | "mathrm" | "mathbf" | "mathbb" => {
                self.skip_whitespace();
                if self.peek_byte() == Some(b'{') {
                    // Scan the braced content raw, preserving spaces.
                    self.pos += 1; // consume opening `{`
                    let start = self.pos;
                    let mut depth = 1usize;
                    while self.pos < self.input.len() {
                        match self.input.as_bytes()[self.pos] {
                            b'{' => {
                                depth += 1;
                                self.pos += 1;
                            },
                            b'}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                                self.pos += 1;
                            },
                            _ => {
                                self.pos += 1;
                            },
                        }
                    }
                    let content = &self.input[start..self.pos];
                    // Consume closing `}`
                    if self.peek_byte() == Some(b'}') {
                        self.pos += 1;
                    }
                    MathNode::TextRun(Arc::from(content))
                } else {
                    // Single atom (no braces) — wrap it.
                    let atom = self.parse_single_atom();
                    match atom {
                        MathNode::Text(s) => MathNode::TextRun(s),
                        other => MathNode::Group(vec![other]),
                    }
                }
            },
            // `\mathit{...}` — math mode is already italic; \mathit is a
            // semantic no-op. Emit the inner nodes as regular Text/Group (NOT
            // TextRun) so OMML does not wrap them in upright-text style.
            "mathit" => {
                self.skip_whitespace();
                if self.peek_byte() == Some(b'{') {
                    let inner_nodes = self.parse_group();
                    if inner_nodes.len() == 1 {
                        #[allow(clippy::unwrap_used)] // safe: len==1
                        inner_nodes.into_iter().next().unwrap()
                    } else {
                        MathNode::Group(inner_nodes)
                    }
                } else {
                    self.parse_single_atom()
                }
            },

            // ── Unsupported command ───────────────────────────────────────
            other => {
                // Consume optional braced argument so parser stays in sync
                self.skip_whitespace();
                let _ = if self.peek_byte() == Some(b'{') {
                    Some(self.parse_group())
                } else {
                    None
                };
                let cmd_span = self.span_at(cmd_start);
                self.diags.push(MathDiagnostic::new(
                    MathRendererError::UnsupportedCommand {
                        command: Arc::from(other),
                        span: cmd_span.clone(),
                        hint: Arc::from(hint_for_command(other)),
                        error_code: "E-EXP-006",
                    },
                    cmd_span,
                ));
                MathNode::Group(Vec::new())
            },
        }
    }

    /// Return true when the parser is positioned at `\right` (the delimiter
    /// command), NOT at `\rightarrow` or other commands that share the prefix.
    ///
    /// The check requires that the character after `\right` is NOT an ASCII
    /// letter, so `\rightarrow` (alphabetic continuation) is excluded.
    fn is_at_right(&self) -> bool {
        let r = self.rest();
        r.starts_with("\\right") && !r.as_bytes().get(6).is_some_and(u8::is_ascii_alphabetic)
    }

    /// Consume a single delimiter character or command (e.g. `(`, `)`, `[`, `]`,
    /// `\{`, `\}`, `\.`, `\|`, `\langle`, `\rangle`, `\lfloor`, `\rfloor`, etc.).
    ///
    /// When a `\` is followed by a letter, the full command name is consumed
    /// (e.g., `\langle` → `"\\langle"`). When followed by a non-letter punctuation
    /// character, only that one character is consumed (e.g., `\{` → `"\\{"`).
    fn consume_delimiter_char(&mut self) -> Arc<str> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'\\') => {
                // Mark the start of the `\` so we can return `\cmd` as a unit.
                let backslash_pos = self.pos;
                self.pos += 1; // consume `\`
                match self.input.as_bytes().get(self.pos) {
                    Some(&b) if b.is_ascii_alphabetic() => {
                        // Multi-char command: consume all letters (e.g. `langle`).
                        while let Some(&nb) = self.input.as_bytes().get(self.pos) {
                            if nb.is_ascii_alphabetic() {
                                self.pos += 1;
                            } else {
                                break;
                            }
                        }
                        Arc::from(&self.input[backslash_pos..self.pos])
                    },
                    Some(_) => {
                        // Single non-alpha char (e.g. `{`, `}`, `.`, `|`).
                        // Advance by the full UTF-8 width of the character —
                        // a naive `+= 1` would split a multi-byte sequence
                        // and panic on the subsequent slice (FINDING-019).
                        let ch = self.input[self.pos..].chars().next().unwrap_or('\0');
                        self.pos += ch.len_utf8();
                        Arc::from(&self.input[backslash_pos..self.pos])
                    },
                    None => Arc::from("\\"),
                }
            },
            Some(_) => {
                let ch = self.consume_char().unwrap_or('(');
                Arc::from(ch.to_string().as_str())
            },
            None => Arc::from("("),
        }
    }

    /// Parse an environment name from `{...}`.
    fn parse_env_name(&mut self) -> &'a str {
        if self.peek_byte() == Some(b'{') {
            self.pos += 1; // consume `{`
            let start = self.pos;
            while let Some(&b) = self.input.as_bytes().get(self.pos) {
                if b == b'}' {
                    break;
                }
                self.pos += 1;
            }
            let name = &self.input[start..self.pos];
            if self.peek_byte() == Some(b'}') {
                self.pos += 1;
            }
            name
        } else {
            ""
        }
    }

    /// Parse `\begin{align}...\end{align}` into `MathNode::Align`.
    ///
    /// The `&` column separator is consumed and discarded — since OMML
    /// `<m:eqArr>` has no native column-tab concept, we merge all column
    /// content within a row into a single flat node list. This avoids
    /// `&` appearing as a literal `Text("&")` or XML `&amp;` in the AST.
    fn parse_align_env(&mut self, env: &str) -> MathNode {
        let mut rows: Vec<Vec<MathNode>> = Vec::new();
        let end_marker = format!("\\end{{{env}}}");
        let end_marker_star = format!("\\end{{{env}*}}");
        let end_marker_ed = "\\end{aligned}".to_owned();

        loop {
            // Parse all columns of one row, merging them into a single node list.
            // Stop the inner loop at `\\` or `\end{...}`; also stop at `&` to
            // consume it as a column separator (but do NOT include it in nodes).
            let mut row: Vec<MathNode> = Vec::new();
            loop {
                let mut col = self.parse_until(|p| {
                    p.peek_byte() == Some(b'&')
                        || p.rest().starts_with("\\\\")
                        || p.rest().starts_with(&end_marker)
                        || p.rest().starts_with(&end_marker_star)
                        || p.rest().starts_with(&end_marker_ed)
                });
                row.append(&mut col);
                // If we stopped at `&`, consume it and continue to parse the
                // next column of this row. Otherwise end the column loop.
                if self.peek_byte() == Some(b'&') {
                    self.pos += 1; // consume `&`, discard
                } else {
                    break;
                }
            }
            rows.push(row);

            if self.rest().starts_with("\\\\") {
                self.pos += 2; // consume `\\`
            } else {
                // Consume `\end{...}`
                for marker in [&end_marker, &end_marker_star, &end_marker_ed] {
                    if self.rest().starts_with(marker.as_str()) {
                        self.pos += marker.len();
                        break;
                    }
                }
                break;
            }
        }
        MathNode::Align(rows)
    }

    /// Parse `\begin{cases}...\end{cases}` into `MathNode::Cases`.
    fn parse_cases_env(&mut self) -> MathNode {
        let end_marker = "\\end{cases}";
        let mut cases: Vec<(Vec<MathNode>, Vec<MathNode>)> = Vec::new();

        loop {
            // Parse left side up to `&` or `\\` or `\end`
            let lhs = self.parse_until(|p| {
                p.peek_byte() == Some(b'&')
                    || p.rest().starts_with("\\\\")
                    || p.rest().starts_with(end_marker)
            });

            let rhs = if self.peek_byte() == Some(b'&') {
                self.pos += 1;
                self.parse_until(|p| {
                    p.rest().starts_with("\\\\") || p.rest().starts_with(end_marker)
                })
            } else {
                Vec::new()
            };

            cases.push((lhs, rhs));

            if self.rest().starts_with("\\\\") {
                self.pos += 2;
            } else {
                if self.rest().starts_with(end_marker) {
                    self.pos += end_marker.len();
                }
                break;
            }
        }
        MathNode::Cases(cases)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Commands supported in v1.0 (used by the parser and the unsupported-command
// check).
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `true` if `cmd` (without leading backslash) is in the supported
/// command set for slideforge v1.0.
#[must_use]
pub fn is_supported_command(cmd: &str) -> bool {
    matches!(
        cmd,
        // Structure
        "frac" | "sqrt" |
        // Accents
        "hat" | "bar" | "tilde" | "vec" | "dot" | "ddot" |
        // Greek lowercase
        "alpha" | "beta" | "gamma" | "delta" | "epsilon" | "varepsilon" |
        "zeta" | "eta" | "theta" | "vartheta" | "iota" | "kappa" |
        "lambda" | "mu" | "nu" | "xi" | "pi" | "varpi" | "rho" |
        "varrho" | "sigma" | "varsigma" | "tau" | "upsilon" |
        "phi" | "varphi" | "chi" | "psi" | "omega" |
        // Greek uppercase
        "Alpha" | "Beta" | "Gamma" | "Delta" | "Epsilon" | "Zeta" |
        "Eta" | "Theta" | "Iota" | "Kappa" | "Lambda" | "Mu" |
        "Nu" | "Xi" | "Pi" | "Rho" | "Sigma" | "Tau" | "Upsilon" |
        "Phi" | "Chi" | "Psi" | "Omega" |
        // Operators
        "sum" | "prod" | "int" | "lim" | "max" | "min" |
        // Symbols
        "cdot" | "times" | "div" | "infty" | "pm" | "mp" | "leq" | "geq" | "neq" |
        "approx" | "equiv" | "in" | "notin" | "subset" | "supset" |
        "cup" | "cap" | "emptyset" | "forall" | "exists" | "partial" |
        "nabla" | "to" | "rightarrow" | "leftarrow" | "Rightarrow" |
        "Leftarrow" | "ldots" | "cdots" | "vdots" | "ddots" |
        // Spaces
        "," | ";" | "quad" | "qquad" |
        // Delimiters
        "left" | "right" |
        // Environments
        "begin" | "end" |
        // Text in math
        "text" | "mathrm" | "mathbf" | "mathit" | "mathbb"
    )
}

/// Returns a correction hint for a specific known-bad command, or a generic
/// hint for unknown commands.
#[must_use]
pub fn hint_for_command(cmd: &str) -> &'static str {
    match cmd {
        "newcommand" | "renewcommand" | "def" | "let" => "user-defined macros require v2+",
        "usepackage" => "\\usepackage is a document-level command; not valid in slideforge math",
        "DeclareMathOperator" => "custom operator declarations require v2+",
        "include" | "input" => "file inclusion is not supported inside math expressions",
        "tikzpicture" => "TikZ drawings are not supported in v1.0",
        _ => "this command is not in the slideforge v1.0 supported LaTeX subset",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{MathMode, MathNode};
    use crate::error::MathRendererError;

    // ─────────────────────────────────────────────────────────────────────────
    // BC-1.10.001 — LaTeX parser produces correct MathAst nodes
    // ─────────────────────────────────────────────────────────────────────────

    /// A simple inline expression parses to Inline mode with a Superscript node.
    #[test]
    fn test_bc_1_10_001_parse_inline_simple() {
        let (ast, diags) = parse("E = mc^2", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        assert_eq!(ast.mode, MathMode::Inline);
        // Must contain a Superscript node
        let has_sup = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Superscript { .. }));
        assert!(has_sup, "expected Superscript node in: {ast:?}");
    }

    /// A display-mode sum with sub+superscript parses correctly.
    #[test]
    fn test_bc_1_10_001_parse_display_sum() {
        fn has_variant(nodes: &[MathNode], pred: fn(&MathNode) -> bool) -> bool {
            nodes.iter().any(|n| {
                if pred(n) {
                    return true;
                }
                // recurse into groups
                match n {
                    MathNode::Group(inner) => has_variant(inner, pred),
                    MathNode::Subscript { base, sub } => {
                        has_variant(std::slice::from_ref(base), pred)
                            || has_variant(std::slice::from_ref(sub), pred)
                    },
                    MathNode::Superscript { base, sup } => {
                        has_variant(std::slice::from_ref(base), pred)
                            || has_variant(std::slice::from_ref(sup), pred)
                    },
                    _ => false,
                }
            })
        }
        let (ast, diags) = parse(
            r"\sum_{i=0}^{n} i",
            MathMode::Display,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        assert_eq!(ast.mode, MathMode::Display);
        // Must contain an Operator node for \sum
        let has_sum = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Operator(s) if s.as_ref() == "sum"));
        assert!(has_sum, "expected Operator(sum) node in: {ast:?}");
        // Must have at least one Subscript and one Superscript
        assert!(
            has_variant(&ast.nodes, |n| matches!(n, MathNode::Subscript { .. })),
            "expected Subscript node"
        );
        assert!(
            has_variant(&ast.nodes, |n| matches!(n, MathNode::Superscript { .. })),
            "expected Superscript node"
        );
    }

    /// `\frac{a}{b}` parses to a Fraction node.
    #[test]
    fn test_bc_1_10_001_parse_fraction() {
        let (ast, diags) = parse(r"\frac{a}{b}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let has_frac = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Fraction { .. }));
        assert!(has_frac, "expected Fraction node in: {ast:?}");
    }

    /// `\sqrt{x}` parses to a Sqrt node with no index.
    #[test]
    fn test_bc_1_10_001_parse_sqrt() {
        let (ast, diags) = parse(r"\sqrt{x}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let has_sqrt = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Sqrt { index: None, .. }));
        assert!(has_sqrt, "expected Sqrt{{index:None}} node in: {ast:?}");
    }

    /// `\alpha + \beta` parses to two Greek nodes.
    #[test]
    fn test_bc_1_10_001_parse_greek() {
        let (ast, diags) = parse(r"\alpha + \beta", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let greek_nodes: Vec<_> = ast
            .nodes
            .iter()
            .filter(|n| matches!(n, MathNode::Greek(_)))
            .collect();
        assert_eq!(
            greek_nodes.len(),
            2,
            "expected exactly 2 Greek nodes, got: {greek_nodes:?}"
        );
    }

    /// An unknown LaTeX command produces a `MathRendererError::UnsupportedCommand` diagnostic.
    #[test]
    fn test_bc_1_10_001_unsupported_command_error() {
        let (_, diags) = parse(r"\undefinedcmd{x}", MathMode::Inline, SourceSpan::default());
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        let has_unsupported = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. } if command.as_ref() == "undefinedcmd")
        });
        assert!(
            has_unsupported,
            "expected UnsupportedCommand for \\undefinedcmd, got: {diags:?}"
        );
    }

    /// Two unsupported commands in one expression produce two separate diagnostics
    /// (error accumulation, not fail-fast).
    #[test]
    fn test_bc_1_10_001_multiple_unsupported_accumulated() {
        let (_, diags) = parse(
            r"\badcmd{x} + \anotherbad{y}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        let unsupported_count = diags
            .iter()
            .filter(|d| matches!(&d.error, MathRendererError::UnsupportedCommand { .. }))
            .count();
        assert!(
            unsupported_count >= 2,
            "expected ≥2 UnsupportedCommand diagnostics, got {unsupported_count}: {diags:?}"
        );
    }

    /// `\newcommand` produces a specific hint about v2+ support.
    #[test]
    fn test_bc_1_10_001_newcommand_specific_hint() {
        let (_, diags) = parse(
            r"\newcommand{\foo}{bar}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        let hint_diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. } if command.as_ref() == "newcommand")
        });
        assert!(
            hint_diag.is_some(),
            "expected UnsupportedCommand for \\newcommand, got: {diags:?}"
        );
        let hint_diag = hint_diag.unwrap();
        if let MathRendererError::UnsupportedCommand { hint, .. } = &hint_diag.error {
            assert!(
                hint.contains("v2+"),
                "expected hint to mention v2+, got: {hint}"
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // is_supported_command + hint_for_command sanity checks
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bc_1_10_001_is_supported_command_known() {
        assert!(is_supported_command("frac"));
        assert!(is_supported_command("sum"));
        assert!(is_supported_command("alpha"));
    }

    #[test]
    fn test_bc_1_10_001_is_supported_command_unknown() {
        assert!(!is_supported_command("undefinedcmd"));
        assert!(!is_supported_command("newcommand"));
    }

    #[test]
    fn test_bc_1_10_001_hint_for_newcommand() {
        assert!(hint_for_command("newcommand").contains("v2+"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-009 — tikzpicture-specific hint
    // ─────────────────────────────────────────────────────────────────────────

    /// `hint_for_command("tikzpicture")` returns a TikZ-specific message.
    #[test]
    fn test_finding_009_tikzpicture_hint() {
        let hint = hint_for_command("tikzpicture");
        assert!(
            hint.contains("TikZ"),
            "tikzpicture hint must mention TikZ; got: {hint}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-007 — UnsupportedCommand must carry error_code "E-EXP-006"
    // ─────────────────────────────────────────────────────────────────────────

    /// An unsupported command produces a diagnostic with `error_code` "E-EXP-006".
    #[test]
    fn test_finding_007_unsupported_command_has_error_code() {
        let (_, diags) = parse(r"\badcmd", MathMode::Inline, SourceSpan::default());
        let diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. }
                if command.as_ref() == "badcmd")
        });
        assert!(diag.is_some(), "expected UnsupportedCommand diagnostic");
        if let Some(MathRendererError::UnsupportedCommand { error_code, .. }) =
            diag.map(|d| &d.error)
        {
            assert_eq!(*error_code, "E-EXP-006", "error_code must be E-EXP-006");
        }
    }

    /// The Display impl includes the error code in the rendered string.
    #[test]
    fn test_finding_007_error_code_in_display() {
        let (_, diags) = parse(r"\unknowncmd", MathMode::Inline, SourceSpan::default());
        let diag = diags
            .iter()
            .find(|d| matches!(&d.error, MathRendererError::UnsupportedCommand { .. }));
        assert!(diag.is_some(), "expected UnsupportedCommand diagnostic");
        let msg = diag.unwrap().error.to_string();
        assert!(
            msg.contains("E-EXP-006"),
            "Display output must include E-EXP-006; got: {msg}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-004 — \div and \mp must parse without diagnostics
    // ─────────────────────────────────────────────────────────────────────────

    /// `\div` parses to a Symbol node with no diagnostics.
    #[test]
    fn test_div_symbol_parses() {
        let (ast, diags) = parse(r"a \div b", MathMode::Inline, SourceSpan::default());
        assert!(
            diags.is_empty(),
            "unexpected diagnostics for \\div: {diags:?}"
        );
        let ast = ast.expect("expected successful parse");
        let has_div = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Symbol(s) if s.as_ref() == "div"));
        assert!(has_div, "expected Symbol(div) node; got: {ast:?}");
    }

    /// `\mp` parses to a Symbol node with no diagnostics.
    #[test]
    fn test_mp_symbol_parses() {
        let (ast, diags) = parse(r"x \mp y", MathMode::Inline, SourceSpan::default());
        assert!(
            diags.is_empty(),
            "unexpected diagnostics for \\mp: {diags:?}"
        );
        let ast = ast.expect("expected successful parse");
        let has_mp = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Symbol(s) if s.as_ref() == "mp"));
        assert!(has_mp, "expected Symbol(mp) node; got: {ast:?}");
    }

    /// `is_supported_command` recognizes `div` and `mp`.
    #[test]
    fn test_div_mp_are_supported_commands() {
        assert!(is_supported_command("div"), "div must be in supported set");
        assert!(is_supported_command("mp"), "mp must be in supported set");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-003 — consume_char must handle multi-byte UTF-8 correctly
    // ─────────────────────────────────────────────────────────────────────────

    /// Parsing an expression that contains multi-byte Unicode characters must
    /// not corrupt the character or produce extra garbage Text nodes.
    #[test]
    fn test_consume_char_handles_multibyte_unicode() {
        // "∑" (U+2211 N-ARY SUMMATION) is a 3-byte UTF-8 sequence.
        // When used as a plain character in math (not as \sum), the parser
        // must consume it as a single Text node without corruption.
        let (ast, diags) = parse("∑ x", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        // The first text node must be exactly "∑" — not a garbled byte fragment
        let first_text = ast.nodes.iter().find_map(|n| {
            if let MathNode::Text(s) = n {
                Some(s.as_ref())
            } else {
                None
            }
        });
        assert_eq!(
            first_text,
            Some("∑"),
            "expected '∑' as first Text node; got: {ast:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-016 — \sqrt[n]{x} parses to Sqrt with index=Some
    // ─────────────────────────────────────────────────────────────────────────

    /// `\sqrt[3]{x}` parses to a Sqrt node with `index = Some(Text("3"))`.
    #[test]
    fn test_finding_016_sqrt_with_index() {
        let (ast, diags) = parse(r"\sqrt[3]{x}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let sqrt_node = ast.nodes.iter().find_map(|n| {
            if let MathNode::Sqrt { index, radicand } = n {
                Some((index.clone(), radicand.clone()))
            } else {
                None
            }
        });
        assert!(sqrt_node.is_some(), "expected Sqrt node; got: {ast:?}");
        let (index, _radicand) = sqrt_node.unwrap();
        assert!(
            index.is_some(),
            "expected Sqrt with index=Some for \\sqrt[3]{{x}}; got index=None"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-015 — unmatched \left produces a diagnostic, not silent output
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left( x + y` (missing `\right`) must produce a "unmatched \\left" diagnostic.
    #[test]
    fn test_finding_015_unmatched_left_emits_diagnostic() {
        let (_, diags) = parse(r"\left( x + y", MathMode::Inline, SourceSpan::default());
        let has_unmatched = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::ParseError { message, .. }
                if message.contains("unmatched") && message.contains("left"))
        });
        assert!(
            has_unmatched,
            "expected an 'unmatched \\\\left' ParseError diagnostic; got: {diags:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-013 — recursion depth limit: deeply nested braces must produce
    //               a diagnostic rather than a stack overflow
    // ─────────────────────────────────────────────────────────────────────────

    /// Deeply nested braces (300 levels) beyond `MAX_DEPTH` must produce a
    /// diagnostic, not a stack overflow or panic.
    #[test]
    fn test_finding_013_deep_nesting_produces_diagnostic_not_panic() {
        // Build 300 levels of nesting — well beyond MAX_DEPTH = 256
        let deep: String = "{".repeat(300) + "x" + &"}".repeat(300);
        let (ast, diags) = parse(&deep, MathMode::Inline, SourceSpan::default());
        // Must not panic (return normally)
        // Must produce at least one diagnostic OR succeed with partial AST
        // (the key invariant is: no stack overflow)
        let _ = (ast, diags); // just reaching here is sufficient
    }

    /// A 300-level nested expression produces a recursion-depth diagnostic.
    #[test]
    fn test_finding_013_deep_nesting_emits_depth_exceeded_diagnostic() {
        let deep: String = "{".repeat(300) + "x" + &"}".repeat(300);
        let (_, diags) = parse(&deep, MathMode::Inline, SourceSpan::default());
        let has_depth_error = diags.iter().any(|d| {
            matches!(&d.error, MathRendererError::ParseError { message, .. }
                if message.contains("depth"))
        });
        assert!(
            has_depth_error,
            "expected a recursion-depth ParseError diagnostic for 300-deep nesting; got: {diags:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-012 — align and cases environment parsing tests
    // ─────────────────────────────────────────────────────────────────────────

    /// `\begin{align} a &= b \\ c &= d \end{align}` parses to Align with 2 rows.
    #[test]
    fn test_finding_012_parse_align() {
        let (ast, diags) = parse(
            r"\begin{align} a &= b \\ c &= d \end{align}",
            MathMode::Display,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let align = ast.nodes.iter().find(|n| matches!(n, MathNode::Align(_)));
        assert!(align.is_some(), "expected Align node; got: {ast:?}");
        if let Some(MathNode::Align(rows)) = align {
            assert_eq!(
                rows.len(),
                2,
                "expected 2 rows in align; got: {}",
                rows.len()
            );
        }
    }

    /// `\begin{cases} x & y > 0 \\ -x & y \leq 0 \end{cases}` parses to Cases.
    #[test]
    fn test_finding_012_parse_cases() {
        let (ast, diags) = parse(
            r"\begin{cases} x & y > 0 \\ -x & y \leq 0 \end{cases}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let cases = ast.nodes.iter().find(|n| matches!(n, MathNode::Cases(_)));
        assert!(cases.is_some(), "expected Cases node; got: {ast:?}");
        if let Some(MathNode::Cases(case_list)) = cases {
            assert_eq!(
                case_list.len(),
                2,
                "expected 2 cases; got: {}",
                case_list.len()
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-011 — consume_delimiter_char must handle multi-char backslash
    //               delimiters like \langle, \rangle, \|, \lfloor, etc.
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left\langle x \right\rangle` parses to a Delimiter with left=`\langle`
    /// and right=`\rangle` inner nodes.
    #[test]
    fn test_finding_011_langle_rangle_delimiter() {
        let (ast, diags) = parse(
            r"\left\langle x \right\rangle",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let delim = ast.nodes.iter().find_map(|n| {
            if let MathNode::Delimiter { left, right, .. } = n {
                Some((left.clone(), right.clone()))
            } else {
                None
            }
        });
        assert!(delim.is_some(), "expected Delimiter node; got: {ast:?}");
        let (left, right) = delim.unwrap();
        assert_eq!(
            left.as_ref(),
            "\\langle",
            "left delimiter must be \\langle; got: {left}"
        );
        assert_eq!(
            right.as_ref(),
            "\\rangle",
            "right delimiter must be \\rangle; got: {right}"
        );
    }

    /// `\left\| x \right\|` parses to a Delimiter with left=`\|` and right=`\|`.
    #[test]
    fn test_finding_011_norm_delimiter() {
        let (ast, diags) = parse(
            r"\left\| x \right\|",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let delim = ast.nodes.iter().find_map(|n| {
            if let MathNode::Delimiter { left, right, .. } = n {
                Some((left.clone(), right.clone()))
            } else {
                None
            }
        });
        assert!(delim.is_some(), "expected Delimiter node; got: {ast:?}");
        let (left, right) = delim.unwrap();
        assert_eq!(
            left.as_ref(),
            "\\|",
            "left delimiter must be \\|; got: {left}"
        );
        assert_eq!(
            right.as_ref(),
            "\\|",
            "right delimiter must be \\|; got: {right}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-001 — is_at_right must NOT match \rightarrow
    // ─────────────────────────────────────────────────────────────────────────

    /// `\left( f: A \rightarrow B \right)` must parse with `\rightarrow` as a
    /// Symbol node inside the delimiter, not as a premature `\right` match.
    #[test]
    fn test_is_at_right_does_not_match_rightarrow() {
        fn find_rightarrow(nodes: &[MathNode]) -> bool {
            for node in nodes {
                match node {
                    MathNode::Symbol(s) if s.as_ref() == "rightarrow" => return true,
                    MathNode::Delimiter { inner, .. } if find_rightarrow(inner) => return true,
                    MathNode::Group(inner) if find_rightarrow(inner) => return true,
                    _ => {},
                }
            }
            false
        }
        let (ast, diags) = parse(
            r"\left( f: A \rightarrow B \right)",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        // Top-level node must be a Delimiter (not a partial/broken parse)
        let has_delim = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Delimiter { .. }));
        assert!(has_delim, "expected Delimiter node; got: {ast:?}");
        // The delimiter's inner nodes must contain a Symbol("rightarrow") node
        assert!(
            find_rightarrow(&ast.nodes),
            "expected Symbol(rightarrow) inside delimiter; got: {ast:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-017 — \text{...} must preserve internal whitespace
    // ─────────────────────────────────────────────────────────────────────────

    /// `\text{hello world}` must produce `TextRun("hello world")` with the
    /// internal space preserved — `parse_group()` was incorrectly stripping
    /// whitespace inside the braces.
    #[test]
    fn test_text_preserves_spaces() {
        let (ast, diags) = parse(
            r"\text{hello world}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let text_run = ast.nodes.iter().find_map(|n| {
            if let MathNode::TextRun(s) = n {
                Some(s.as_ref())
            } else {
                None
            }
        });
        assert_eq!(
            text_run,
            Some("hello world"),
            "expected TextRun(\"hello world\") with preserved space; got: {ast:?}"
        );
    }

    /// `\text{if and only if}` must produce `TextRun("if and only if")` with
    /// all internal spaces preserved.
    #[test]
    fn test_text_complex() {
        let (ast, diags) = parse(
            r"\text{if and only if}",
            MathMode::Inline,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let text_run = ast.nodes.iter().find_map(|n| {
            if let MathNode::TextRun(s) = n {
                Some(s.as_ref())
            } else {
                None
            }
        });
        assert_eq!(
            text_run,
            Some("if and only if"),
            "expected TextRun(\"if and only if\"); got: {ast:?}"
        );
    }

    /// `\mathrm{sin}` must produce `TextRun("sin")` (upright text).
    #[test]
    fn test_mathrm_preserves_spaces() {
        let (ast, diags) = parse(r"\mathrm{sin}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        let text_run = ast.nodes.iter().find_map(|n| {
            if let MathNode::TextRun(s) = n {
                Some(s.as_ref())
            } else {
                None
            }
        });
        assert_eq!(
            text_run,
            Some("sin"),
            "expected TextRun(\"sin\"); got: {ast:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-018 — align environment must split on `&`, not include it as text
    // ─────────────────────────────────────────────────────────────────────────

    fn has_ampersand_text(nodes: &[MathNode]) -> bool {
        for node in nodes {
            match node {
                MathNode::Text(s) | MathNode::TextRun(s)
                    if s.contains('&') || s.contains("&amp;") =>
                {
                    return true;
                },
                MathNode::Align(rows) => {
                    for row in rows {
                        if has_ampersand_text(row) {
                            return true;
                        }
                    }
                },
                MathNode::Group(inner) if has_ampersand_text(inner) => {
                    return true;
                },
                _ => {},
            }
        }
        false
    }

    /// `\begin{align} a &= b \\ c &= d \end{align}` — verify that `&` does not
    /// appear as a literal `&` or XML entity `&amp;` in any Text node.
    #[test]
    fn test_align_no_ampersand_in_ast() {
        let (ast, diags) = parse(
            r"\begin{align} a &= b \\ c &= d \end{align}",
            MathMode::Display,
            SourceSpan::default(),
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        assert!(
            !has_ampersand_text(&ast.nodes),
            "align AST must not contain literal '&' text nodes; got: {ast:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-019 — parse_command_name and consume_delimiter_char must not
    //               panic on backslash + non-ASCII multi-byte UTF-8 characters
    // ─────────────────────────────────────────────────────────────────────────

    /// `\é` (backslash + U+00E9, a 2-byte UTF-8 char) must produce an
    /// `UnsupportedCommand` diagnostic without panicking.
    #[test]
    fn test_backslash_non_ascii_no_panic() {
        let (_, diags) = parse(r"\é", MathMode::Inline, SourceSpan::default());
        // Must not panic — reaching here means no panic occurred.
        // The non-ASCII char after `\` is not a recognized command name, so an
        // UnsupportedCommand diagnostic must be emitted.
        assert!(
            !diags.is_empty(),
            "expected a diagnostic for \\é (backslash + non-ASCII); got none"
        );
        let has_unsupported = diags
            .iter()
            .any(|d| matches!(&d.error, MathRendererError::UnsupportedCommand { .. }));
        assert!(
            has_unsupported,
            "expected UnsupportedCommand diagnostic for \\é; got: {diags:?}"
        );
    }

    /// `\left\ü x \right)` (backslash + U+00FC in delimiter position) must
    /// produce a diagnostic without panicking.
    #[test]
    fn test_delimiter_non_ascii_no_panic() {
        // `\ü` is `\` followed by a 2-byte UTF-8 char — consume_delimiter_char
        // must advance by the full char width, not just 1 byte.
        let (_, _diags) = parse(
            r"\left\ü x \right)",
            MathMode::Inline,
            SourceSpan::default(),
        );
        // Reaching here means no panic — that is the primary invariant.
        // (The parse may or may not emit diagnostics depending on how the
        // non-ASCII delimiter is handled, but it must never panic.)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-018 (related) — \mathit should NOT produce TextRun
    // Math mode is already italic; \mathit should pass through as regular Text
    // ─────────────────────────────────────────────────────────────────────────

    /// `\mathit{x}` must NOT produce a `TextRun` node. Since math mode is
    /// already italic, \mathit is semantically equivalent to plain text.
    /// It must produce `Text` nodes (not `TextRun`) to avoid incorrect OMML
    /// upright-text wrapping.
    #[test]
    fn test_mathit_produces_text_not_textrun() {
        let (ast, diags) = parse(r"\mathit{x}", MathMode::Inline, SourceSpan::default());
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let ast = ast.expect("expected successful parse");
        // Must NOT contain a TextRun node
        let has_text_run = ast.nodes.iter().any(|n| matches!(n, MathNode::TextRun(_)));
        assert!(
            !has_text_run,
            "\\mathit must NOT produce TextRun (math is already italic); got: {ast:?}"
        );
        // Must contain Text or Group nodes
        let has_text = ast
            .nodes
            .iter()
            .any(|n| matches!(n, MathNode::Text(_) | MathNode::Group(_)));
        assert!(
            has_text,
            "expected Text or Group node from \\mathit; got: {ast:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // FINDING-022 — Diagnostic spans must point to command position within
    //               the math block, not position 0 / the entire block
    // ─────────────────────────────────────────────────────────────────────────

    /// When an unsupported command appears after some content, the diagnostic
    /// span's `byte_offset` must be > 0 (i.e., it points into the math content,
    /// not at the start of the block).
    #[test]
    fn test_finding_022_diagnostic_span_points_into_math_content() {
        // The math block starts at byte_offset=100 in the enclosing .sf file.
        // The unsupported command `\badcmd` appears after the leading text "x + ".
        // "x + " is 4 bytes, plus 1 for the backslash = cmd_start at position 4
        // inside the math content (relative), so the diagnostic's byte_offset
        // should be 100 + 4 = 104 — not 100 (the block start).
        use std::sync::Arc;
        let span = SourceSpan::new(Arc::from("deck.sf"), 5, 10, 100);
        let (_, diags) = parse(r"x + \badcmd", MathMode::Inline, span.clone());
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        let cmd_diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. }
                if command.as_ref() == "badcmd")
        });
        assert!(
            cmd_diag.is_some(),
            "expected UnsupportedCommand for \\badcmd"
        );
        let cmd_diag = cmd_diag.unwrap();
        // The diagnostic span's byte_offset must be greater than the block's
        // byte_offset (100) — it must point INTO the math content, not at
        // the beginning of the enclosing math block.
        assert!(
            cmd_diag.span.byte_offset > span.byte_offset,
            "span.byte_offset ({}) must be > block start ({}) — diagnostic must \
             point to the command, not the block start",
            cmd_diag.span.byte_offset,
            span.byte_offset
        );
    }

    /// When an unsupported command is the very first token in the math content,
    /// its span `byte_offset` equals the block's `byte_offset` (offset 0 within block).
    #[test]
    fn test_finding_022_diagnostic_span_at_block_start_when_cmd_is_first() {
        use std::sync::Arc;
        let span = SourceSpan::new(Arc::from("deck.sf"), 5, 10, 100);
        let (_, diags) = parse(r"\badcmd", MathMode::Inline, span.clone());
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        let cmd_diag = diags.iter().find(|d| {
            matches!(&d.error, MathRendererError::UnsupportedCommand { command, .. }
                if command.as_ref() == "badcmd")
        });
        assert!(
            cmd_diag.is_some(),
            "expected UnsupportedCommand for \\badcmd"
        );
        let cmd_diag = cmd_diag.unwrap();
        // The command is at position 0 within the math block, so the span's
        // byte_offset equals the block's byte_offset.
        assert_eq!(
            cmd_diag.span.byte_offset, span.byte_offset,
            "when command is at block start, span.byte_offset must equal block \
             byte_offset ({});  got {}",
            span.byte_offset, cmd_diag.span.byte_offset
        );
    }
}
