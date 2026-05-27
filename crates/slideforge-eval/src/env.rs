//! Variable environment for the slideforge expression evaluator.
//!
//! [`Env`] provides a stack of scope frames. The outermost frame holds deck-level
//! variables declared in `vars:` blocks. Inner frames are pushed/popped around
//! `@for` loop bodies, `@let` bindings, and `@with` scopes.
//!
//! Lookup walks from the innermost (most-recently-pushed) frame outward,
//! returning the first match — inner scopes shadow outer scopes.

use std::sync::Arc;

use indexmap::IndexMap;
use slideforge_types::Value;

// ─── Env ─────────────────────────────────────────────────────────────────────

/// A scoped variable environment for expression evaluation.
///
/// The environment is a stack of [`IndexMap`] frames. Each frame maps
/// variable names (`Arc<str>`) to [`Value`]s. Lookup walks inward-to-outward,
/// so inner scopes shadow outer ones.
///
/// # Examples
///
/// ```
/// use slideforge_eval::Env;
/// use slideforge_types::Value;
/// use indexmap::IndexMap;
/// use std::sync::Arc;
///
/// let mut vars = IndexMap::new();
/// vars.insert(Arc::from("n"), Value::Int(42));
/// let env = Env::new(vars);
/// assert_eq!(env.lookup("n"), Some(&Value::Int(42)));
/// assert_eq!(env.lookup("x"), None);
/// ```
#[derive(Debug, Clone)]
pub struct Env {
    /// Stack of scope frames; last element is the innermost scope.
    frames: Vec<IndexMap<Arc<str>, Value>>,
}

impl Env {
    /// Construct a new environment with a single deck-level frame.
    ///
    /// `deck_vars` should contain all variables resolved from the `vars:` block
    /// at the top of the `.sf` file.
    #[must_use]
    pub fn new(deck_vars: IndexMap<Arc<str>, Value>) -> Self {
        Self {
            frames: vec![deck_vars],
        }
    }

    /// Push a new inner scope frame.
    ///
    /// Bindings in `bindings` shadow any same-named variables in outer frames
    /// for the duration of this scope.
    pub fn push_scope(&mut self, bindings: IndexMap<Arc<str>, Value>) {
        self.frames.push(bindings);
    }

    /// Pop the innermost scope frame.
    ///
    /// If only the deck-level frame remains, it is NOT popped — the environment
    /// always retains at least one frame.
    pub fn pop_scope(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Look up a variable by name, searching from innermost to outermost frame.
    ///
    /// Returns `None` if the variable is not defined in any visible scope.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        // Walk from innermost to outermost frame.
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        None
    }

    /// Return a sorted list of all variable names visible in the current scope.
    ///
    /// Duplicates are deduplicated (inner scope wins, but the name is listed
    /// once). The returned list is sorted lexicographically for stable
    /// diagnostic output.
    #[must_use]
    pub fn all_names(&self) -> Vec<Arc<str>> {
        // Collect names in outer-to-inner order, then dedup by name.
        // Use an IndexMap to preserve insertion order while deduplicating.
        let mut seen: IndexMap<Arc<str>, ()> = IndexMap::new();
        for frame in &self.frames {
            for key in frame.keys() {
                seen.insert(key.clone(), ());
            }
        }
        let mut names: Vec<Arc<str>> = seen.into_keys().collect();
        names.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
        names
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_env_with(pairs: &[(&str, Value)]) -> Env {
        let mut vars = IndexMap::new();
        for (k, v) in pairs {
            vars.insert(Arc::from(*k), v.clone());
        }
        Env::new(vars)
    }

    #[test]
    fn test_env_lookup_deck_var() {
        let env = make_env_with(&[("a", Value::Int(1))]);
        assert_eq!(env.lookup("a"), Some(&Value::Int(1)));
    }

    #[test]
    fn test_env_unknown_returns_none() {
        let env = make_env_with(&[("a", Value::Int(1))]);
        assert_eq!(env.lookup("z"), None);
    }

    #[test]
    fn test_env_scope_push_pop() {
        let mut env = make_env_with(&[("a", Value::Int(1))]);

        // Push inner scope with "b".
        let mut inner: IndexMap<Arc<str>, Value> = IndexMap::new();
        inner.insert(Arc::from("b"), Value::Str(Arc::from("x")));
        env.push_scope(inner);

        // Both "a" (outer) and "b" (inner) should be visible.
        assert_eq!(env.lookup("a"), Some(&Value::Int(1)));
        assert_eq!(env.lookup("b"), Some(&Value::Str(Arc::from("x"))));

        // Pop inner scope — "b" should disappear; "a" should remain.
        env.pop_scope();
        assert_eq!(env.lookup("b"), None, "b must not be visible after pop");
        assert_eq!(
            env.lookup("a"),
            Some(&Value::Int(1)),
            "a must persist after pop"
        );
    }

    #[test]
    fn test_env_all_names() {
        let env = make_env_with(&[
            ("alpha", Value::Int(1)),
            ("beta", Value::Bool(true)),
            ("gamma", Value::Null),
        ]);
        let names = env.all_names();
        assert_eq!(names.len(), 3, "all_names must contain all 3 variables");
        // Sorted lexicographically.
        assert_eq!(names[0].as_ref(), "alpha");
        assert_eq!(names[1].as_ref(), "beta");
        assert_eq!(names[2].as_ref(), "gamma");
    }

    #[test]
    fn test_env_inner_scope_shadows_outer() {
        let mut env = make_env_with(&[("x", Value::Int(10))]);
        let mut inner: IndexMap<Arc<str>, Value> = IndexMap::new();
        inner.insert(Arc::from("x"), Value::Int(99));
        env.push_scope(inner);
        // Inner scope must shadow outer.
        assert_eq!(env.lookup("x"), Some(&Value::Int(99)));
        env.pop_scope();
        // After pop, outer value restored.
        assert_eq!(env.lookup("x"), Some(&Value::Int(10)));
    }

    #[test]
    fn test_env_pop_on_single_frame_is_noop() {
        // Popping the only frame must not panic or remove the deck-level frame.
        let mut env = make_env_with(&[("a", Value::Int(1))]);
        env.pop_scope(); // Should be a no-op.
        assert_eq!(env.lookup("a"), Some(&Value::Int(1)));
    }

    #[test]
    fn test_env_all_names_deduplicates_shadowed() {
        let mut env = make_env_with(&[("a", Value::Int(1)), ("b", Value::Int(2))]);
        let mut inner: IndexMap<Arc<str>, Value> = IndexMap::new();
        // "a" is in both frames — all_names must list it once.
        inner.insert(Arc::from("a"), Value::Int(99));
        inner.insert(Arc::from("c"), Value::Int(3));
        env.push_scope(inner);

        let names = env.all_names();
        let count_a = names.iter().filter(|n| n.as_ref() == "a").count();
        assert_eq!(
            count_a, 1,
            "shadowed variable 'a' must appear only once in all_names"
        );
        assert!(names.contains(&Arc::from("b")));
        assert!(names.contains(&Arc::from("c")));
    }

    #[test]
    fn test_env_clone() {
        let env = make_env_with(&[("n", Value::Int(7))]);
        let env2 = env.clone();
        assert_eq!(env2.lookup("n"), Some(&Value::Int(7)));
    }
}
