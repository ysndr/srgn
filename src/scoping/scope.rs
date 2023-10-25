use crate::scoping::scope::Scope::{In, Out};
use itertools::Itertools;
use log::{debug, trace};
use std::{borrow::Cow, ops::Range};

/// Indicates whether a given string part is in scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope<'viewee, T> {
    /// The given part is in scope for processing.
    In(T),
    /// The given part is out of scope for processing.
    ///
    /// Treated as immutable, view-only.
    Out(&'viewee str),
}

/// A read-only scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScope<'viewee>(pub Scope<'viewee, &'viewee str>);

/// Multiple read-only scopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ROScopes<'viewee>(pub Vec<ROScope<'viewee>>);

/// A read-write scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScope<'viewee>(pub Scope<'viewee, Cow<'viewee, str>>);

/// Multiple read-write scopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RWScopes<'viewee>(pub Vec<RWScope<'viewee>>);

impl<'viewee> ROScope<'viewee> {
    /// Check whether the scope is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let s: &str = self.into();
        s.is_empty()
    }
}

impl<'viewee> ROScopes<'viewee> {
    /// Construct a new instance from the given raw ranges.
    ///
    /// The passed `input` will be traversed according to `ranges`: all specified
    /// `ranges` are taken as [`In`] scope, everything not covered by a range is [`Out`]
    /// of scope.
    ///
    /// ## Panics
    ///
    /// Panics if the given `ranges` contain indices out-of-bounds for `input`.
    #[must_use]
    pub fn from_raw_ranges(input: &'viewee str, ranges: Vec<Range<usize>>) -> Self {
        let mut scopes = Vec::with_capacity(ranges.len());

        let mut last_end = 0;
        for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
            scopes.push(ROScope(Out(&input[last_end..start])));
            scopes.push(ROScope(In(&input[start..end])));
            last_end = end;
        }

        if last_end < input.len() {
            scopes.push(ROScope(Out(&input[last_end..])));
        }

        scopes.retain(|s| !s.is_empty());

        debug!("Scopes: {:?}", scopes);

        ROScopes(scopes)
    }

    /// Inverts the scopes: what was previously [`In`] is now [`Out`], and vice versa.
    #[must_use]
    pub fn invert(self) -> Self {
        trace!("Inverting scopes: {:?}", self.0);
        let scopes = self
            .0
            .into_iter()
            .map(|s| match s {
                ROScope(In(s)) => ROScope(Out(s)),
                ROScope(Out(s)) => ROScope(In(s)),
            })
            .collect();
        trace!("Inverted scopes: {:?}", scopes);

        Self(scopes)
    }
}

impl<'viewee> From<&'viewee ROScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee ROScope) -> Self {
        match s.0 {
            In(s) | Out(s) => s,
        }
    }
}

impl<'viewee> From<ROScope<'viewee>> for RWScope<'viewee> {
    fn from(s: ROScope<'viewee>) -> Self {
        match s.0 {
            In(s) => RWScope(In(Cow::Borrowed(s))),
            Out(s) => RWScope(Out(s)),
        }
    }
}

impl<'viewee> From<&'viewee RWScope<'viewee>> for &'viewee str {
    /// Get the underlying string slice.
    ///
    /// All variants contain such a slice, so this is a convenient method.
    fn from(s: &'viewee RWScope) -> Self {
        match &s.0 {
            In(s) => s,
            Out(s) => s,
        }
    }
}
