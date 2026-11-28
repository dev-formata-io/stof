//
// Copyright 2025 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//! Semantic versions (the `ver` type): precedence, requirements, and compatibility, as semver.org and Cargo define them.
//!
//! - **Precedence:** major, minor, and patch as numbers; then a pre-release comes before its release (1.0.0-rc.1 <
//!   1.0.0), compared by its dot-separated identifiers (numbers as numbers, below words, and more identifiers after
//!   fewer). Build metadata doesn't count. A wildcard part (`*`) matches any number.
//! - **Requirements** (`Ver.matches`), as Cargo's: `^1.2.3` (and a bare `1.2.3`), `~1.2`, `=1.2.3`, `>1.2`, `>=1.2`,
//!   `<2`, `<=2.1`, `1.2.*`, `*`, separated by commas (all must match). A pre-release matches only a requirement with
//!   a pre-release of its own version (1.5.0-rc.2 matches `>=1.5.0-rc.1`, not `>=1.4.0`).
//! - **Compatible** (packages): the newer can stand in for the older, as `^older` matches it.

use std::{cmp::Ordering, fmt};
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::runtime::Val;


/*****************************************************************************
 * Precedence.
 *****************************************************************************/

/// Compare pre-releases: a release (None) comes after any pre-release; pre-releases by their identifiers.
pub fn cmp_pre(a: Option<&str>, b: Option<&str>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => {
            let mut a_parts = a.split('.');
            let mut b_parts = b.split('.');
            loop {
                match (a_parts.next(), b_parts.next()) {
                    (None, None) => return Ordering::Equal,
                    (None, Some(_)) => return Ordering::Less, // fewer identifiers first
                    (Some(_), None) => return Ordering::Greater,
                    (Some(a), Some(b)) => {
                        let order = match (numeric(a), numeric(b)) {
                            (Some(a), Some(b)) => a.cmp(&b),
                            (Some(_), None) => Ordering::Less, // numbers before words
                            (None, Some(_)) => Ordering::Greater,
                            (None, None) => a.cmp(b),
                        };
                        if order != Ordering::Equal { return order; }
                    },
                }
            }
        },
    }
}

/// A pre-release identifier as a number, if it's all digits.
fn numeric(part: &str) -> Option<u64> {
    if !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) { part.parse().ok() } else { None }
}

/// Precedence of two versions (major, minor, patch, pre-release), a wildcard part (negative) matching any number.
pub fn cmp_precedence(a: (i32, i32, i32, Option<&str>), b: (i32, i32, i32, Option<&str>)) -> Ordering {
    for (x, y) in [(a.0, b.0), (a.1, b.1), (a.2, b.2)] {
        if x < 0 || y < 0 { continue; } // a wildcard
        let order = x.cmp(&y);
        if order != Ordering::Equal { return order; }
    }
    cmp_pre(a.3, b.3)
}

/// A total order for `ver` values (sorting, sets, and maps): precedence by number (a wildcard as -1), then the
/// pre-release and build text, so only identical versions are equal.
pub fn total_cmp(a: (i32, i32, i32, Option<&str>, Option<&str>), b: (i32, i32, i32, Option<&str>, Option<&str>)) -> Ordering {
    (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2))
        .then_with(|| cmp_pre(a.3, b.3))
        .then_with(|| a.3.cmp(&b.3))
        .then_with(|| a.4.cmp(&b.4))
}


/*****************************************************************************
 * Parsing.
 *****************************************************************************/

/// Parse a version from text (Ex. a string cast to `ver`): `1.2.3`, an optional `-pre.release` and `+build.metadata`,
/// a leading `v` ("v1.2.3", as tags have), and wildcard parts (`1.2.*`, `1.*`, `*`; `x` works too). Parts are
/// numbers; pre-release and build identifiers are letters, digits, and `-`, separated by dots.
pub fn parse_ver(text: &str) -> Option<Val> {
    let text = text.trim();
    let text = text.strip_prefix(['v', 'V']).unwrap_or(text);
    let (text, build) = match text.split_once('+') {
        Some((text, build)) => (text, Some(identifiers(build)?)),
        None => (text, None),
    };
    let (core, pre) = match text.split_once('-') {
        Some((core, pre)) => (core, Some(identifiers(pre)?)),
        None => (text, None),
    };
    let mut parts = [-1i32; 3];
    let mut wild = false;
    let pieces: Vec<&str> = core.split('.').collect();
    if pieces.is_empty() || pieces.len() > 3 { return None; }
    for (i, piece) in pieces.iter().enumerate() {
        if matches!(*piece, "*" | "x" | "X") { wild = true; continue; }
        if wild || piece.is_empty() || !piece.bytes().all(|b| b.is_ascii_digit()) { return None; } // a number after a wildcard
        parts[i] = piece.parse().ok()?;
    }
    if pieces.len() < 3 && !wild { return None; } // "1.2" is a number, not a version
    if wild && (pre.is_some() || build.is_some()) { return None; }
    Some(Val::Ver(parts[0], parts[1], parts[2], pre, build))
}

/// Dot-separated identifiers (letters, digits, and '-'), none empty.
fn identifiers(text: &str) -> Option<ArcStr> {
    if text.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')) {
        Some(text.into())
    } else {
        None
    }
}

/// A value as a version, for comparing: a `ver`, or a string that parses as one.
pub fn as_ver(val: &Val) -> Option<Val> {
    match val {
        Val::Ver(..) => Some(val.clone()),
        Val::Str(text) => parse_ver(text),
        _ => None,
    }
}


/*****************************************************************************
 * Versions (Rust).
 *****************************************************************************/

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A version without wildcards (Ex. a package's): MAJOR.MINOR.PATCH and an optional pre-release. Build metadata is
/// dropped, as semver ignores it.
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    /// The pre-release (Ex. "rc.1"), if it's one.
    pub pre: Option<ArcStr>,
}
impl Version {
    /// Parse a version (see `parse_ver`), without wildcards.
    pub fn parse(text: &str) -> Result<Self, String> {
        match parse_ver(text) {
            Some(Val::Ver(major, minor, patch, pre, _build)) if major >= 0 && minor >= 0 && patch >= 0 => {
                Ok(Self { major: major as u64, minor: minor as u64, patch: patch as u64, pre })
            },
            _ => Err(format!("'{text}' isn't a version (Ex. 1.4.0: MAJOR.MINOR.PATCH, with an optional -pre.release)")),
        }
    }

    /// Is this a pre-release?
    pub fn is_pre(&self) -> bool {
        self.pre.is_some()
    }

    /// The part that has to match for two versions to be compatible: the major version, or for 0.x the minor
    /// version, or for 0.0.x the patch (Cargo's caret rules).
    pub fn compat_key(&self) -> (u64, u64, u64) {
        if self.major > 0 { (self.major, 0, 0) }
        else if self.minor > 0 { (0, self.minor, 0) }
        else { (0, 0, self.patch) }
    }

    /// Can the newer of these versions stand in for the older (`^older` matches it)? A pre-release goes only with its
    /// own version's pre-releases and the releases from that version on (Ex. 1.5.0-rc.1 with 1.5.0-rc.2 and 1.6.0,
    /// not 1.4.0), as Cargo matches them.
    pub fn compatible(&self, other: &Self) -> bool {
        let (older, newer) = if self <= other { (self, other) } else { (other, self) };
        let caret = Comparator { op: Op::Caret, major: older.major, minor: Some(older.minor), patch: Some(older.patch), pre: older.pre.clone() };
        caret.matches(newer) && (!newer.is_pre() || caret.allows_pre(newer))
    }

    /// The compatible range as text, for messages (Ex. "1.x", "0.3.x", "0.0.3").
    pub fn series(&self) -> String {
        if self.major > 0 { format!("{}.x", self.major) }
        else if self.minor > 0 { format!("0.{}.x", self.minor) }
        else { format!("0.0.{}", self.patch) }
    }

    /// This version as a `ver` value.
    pub fn to_val(&self) -> Val {
        Val::Ver(self.major as i32, self.minor as i32, self.patch as i32, self.pre.clone(), None)
    }

    /// A `ver` value as a version (not a wildcard one).
    pub fn from_val(val: &Val) -> Option<Self> {
        match val {
            Val::Ver(major, minor, patch, pre, _) if *major >= 0 && *minor >= 0 && *patch >= 0 => {
                Some(Self { major: *major as u64, minor: *minor as u64, patch: *patch as u64, pre: pre.clone() })
            },
            _ => None,
        }
    }
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre { write!(f, "-{pre}")?; }
        Ok(())
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
            .then_with(|| cmp_pre(self.pre.as_deref(), other.pre.as_deref()))
            .then_with(|| self.pre.cmp(&other.pre)) // identical only when the text is ("01" and "1")
    }
}


/*****************************************************************************
 * Requirements.
 *****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op { Exact, Greater, GreaterEq, Less, LessEq, Tilde, Caret, Wildcard }

#[derive(Debug, Clone, PartialEq)]
/// One comparator of a requirement (Ex. `>=1.2`): missing parts are None.
struct Comparator {
    op: Op,
    major: u64,
    minor: Option<u64>,
    patch: Option<u64>,
    pre: Option<ArcStr>,
}
impl Comparator {
    fn parse(text: &str) -> Result<Option<Self>, String> {
        let bad = || format!("'{text}' isn't a version requirement (Ex. ^1.2.3, ~1.2, >=1.0.0, <2.0.0, 1.2.*, or *)");
        let mut rest = text.trim();
        let mut op = None;
        for (prefix, found) in [(">=", Op::GreaterEq), ("<=", Op::LessEq), (">", Op::Greater), ("<", Op::Less), ("=", Op::Exact), ("~", Op::Tilde), ("^", Op::Caret)] {
            if let Some(after) = rest.strip_prefix(prefix) { op = Some(found); rest = after.trim_start(); break; }
        }
        let rest = rest.strip_prefix(['v', 'V']).unwrap_or(rest);
        let rest = rest.split_once('+').map(|(version, _build)| version).unwrap_or(rest); // build: doesn't count
        let (core, pre) = match rest.split_once('-') {
            Some((core, pre)) => (core, Some(identifiers(pre).ok_or_else(bad)?)),
            None => (rest, None),
        };
        let mut parts: [Option<u64>; 3] = [None; 3];
        let mut wild = false;
        let pieces: Vec<&str> = core.split('.').collect();
        if pieces.len() > 3 { return Err(bad()); }
        for (i, piece) in pieces.iter().enumerate() {
            if matches!(*piece, "*" | "x" | "X") { wild = true; continue; }
            if wild || piece.is_empty() || !piece.bytes().all(|b| b.is_ascii_digit()) { return Err(bad()); }
            parts[i] = Some(piece.parse().map_err(|_| bad())?);
        }
        if pre.is_some() && parts[2].is_none() { return Err(bad()); } // a pre-release needs a whole version
        let Some(major) = parts[0] else {
            // "*" (or ">=*"): any version
            return if op.is_none() || op == Some(Op::GreaterEq) || op == Some(Op::LessEq) || op == Some(Op::Exact) { Ok(None) } else { Err(bad()) };
        };
        let op = match op {
            Some(op) => op,
            None if wild => Op::Wildcard,
            None => Op::Caret,
        };
        Ok(Some(Self { op, major, minor: parts[1], patch: parts[2], pre }))
    }

    fn matches(&self, ver: &Version) -> bool {
        match self.op {
            Op::Exact | Op::Wildcard => self.matches_exact(ver),
            Op::Greater => self.matches_greater(ver),
            Op::GreaterEq => self.matches_exact(ver) || self.matches_greater(ver),
            Op::Less => self.matches_less(ver),
            Op::LessEq => self.matches_exact(ver) || self.matches_less(ver),
            Op::Tilde => self.matches_tilde(ver),
            Op::Caret => self.matches_caret(ver),
        }
    }

    fn pre_order(&self, ver: &Version) -> Ordering {
        cmp_pre(ver.pre.as_deref(), self.pre.as_deref())
    }

    fn matches_exact(&self, ver: &Version) -> bool {
        if ver.major != self.major { return false; }
        if self.minor.is_some_and(|minor| ver.minor != minor) { return false; }
        if self.patch.is_some_and(|patch| ver.patch != patch) { return false; }
        self.patch.is_none() || self.pre_order(ver) == Ordering::Equal
    }

    fn matches_greater(&self, ver: &Version) -> bool {
        if ver.major != self.major { return ver.major > self.major; }
        let Some(minor) = self.minor else { return false };
        if ver.minor != minor { return ver.minor > minor; }
        let Some(patch) = self.patch else { return false };
        if ver.patch != patch { return ver.patch > patch; }
        self.pre_order(ver) == Ordering::Greater
    }

    fn matches_less(&self, ver: &Version) -> bool {
        if ver.major != self.major { return ver.major < self.major; }
        let Some(minor) = self.minor else { return false };
        if ver.minor != minor { return ver.minor < minor; }
        let Some(patch) = self.patch else { return false };
        if ver.patch != patch { return ver.patch < patch; }
        self.pre_order(ver) == Ordering::Less
    }

    fn matches_tilde(&self, ver: &Version) -> bool {
        if ver.major != self.major { return false; }
        if self.minor.is_some_and(|minor| ver.minor != minor) { return false; }
        if let Some(patch) = self.patch {
            if ver.patch != patch { return ver.patch > patch; }
        }
        self.pre_order(ver) != Ordering::Less
    }

    fn matches_caret(&self, ver: &Version) -> bool {
        if ver.major != self.major { return false; }
        let Some(minor) = self.minor else { return true };
        let Some(patch) = self.patch else {
            return if self.major > 0 { ver.minor >= minor } else { ver.minor == minor };
        };
        if self.major > 0 {
            if ver.minor != minor { return ver.minor > minor; }
            if ver.patch != patch { return ver.patch > patch; }
        } else if minor > 0 {
            if ver.minor != minor { return false; }
            if ver.patch != patch { return ver.patch > patch; }
        } else if ver.minor != minor || ver.patch != patch {
            return false;
        }
        self.pre_order(ver) != Ordering::Less
    }

    /// May a pre-release match this comparator? Only one of the same version, when the comparator has a pre-release.
    fn allows_pre(&self, ver: &Version) -> bool {
        self.pre.is_some() && self.major == ver.major && self.minor == Some(ver.minor) && self.patch == Some(ver.patch)
    }
}


#[derive(Debug, Clone, PartialEq)]
/// A version requirement (Ex. `>=1.2.0, <2.0.0`): every comparator must match. No comparators (`*`) matches any
/// release.
pub struct VersionReq {
    comparators: Vec<Comparator>,
}
impl VersionReq {
    /// Parse a requirement: comparators separated by commas.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.trim().is_empty() { return Err(format!("'{text}' isn't a version requirement (Ex. ^1.2.3, ~1.2, >=1.0.0, <2.0.0, 1.2.*, or *)")); }
        let mut comparators = Vec::new();
        for part in text.split(',') {
            if let Some(comparator) = Comparator::parse(part)? { comparators.push(comparator); }
        }
        Ok(Self { comparators })
    }

    /// Does this version match? A pre-release matches only when a comparator has a pre-release of its version.
    pub fn matches(&self, ver: &Version) -> bool {
        self.comparators.iter().all(|comparator| comparator.matches(ver))
            && (!ver.is_pre() || self.comparators.iter().any(|comparator| comparator.allows_pre(ver)))
    }
}


#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use super::{cmp_pre, parse_ver, Version, VersionReq};
    use crate::runtime::Val;

    fn v(text: &str) -> Version { Version::parse(text).unwrap() }
    fn req(text: &str) -> VersionReq { VersionReq::parse(text).unwrap() }

    #[test]
    fn parse() {
        assert_eq!(v("1.4.0"), Version { major: 1, minor: 4, patch: 0, pre: None });
        assert_eq!(v("v1.4.0").to_string(), "1.4.0");
        assert_eq!(v(" 0.2.10-rc.1+build.5 ").to_string(), "0.2.10-rc.1");
        assert_eq!(v("1.0.0-alpha-2.x").pre.as_deref(), Some("alpha-2.x"));
        for bad in ["", "1", "1.2", "1.2.3.4", "1.2.x", "a.b.c", "1.2.3-", "1.2.3-rc..1", "1.2.3-r c", "-1.2.3", "1.2.3 4", "1.2.3+", "1.2.3+a..b", "*"] {
            assert!(Version::parse(bad).is_err(), "{bad}");
        }
        assert_eq!(parse_ver("v2.1.0-rc.1+sha.5114f85"), Some(Val::Ver(2, 1, 0, Some("rc.1".into()), Some("sha.5114f85".into()))));
        assert_eq!(parse_ver("1.2.*"), Some(Val::Ver(1, 2, -1, None, None)));
        assert_eq!(parse_ver("1.x"), Some(Val::Ver(1, -1, -1, None, None)));
        assert_eq!(parse_ver("*"), Some(Val::Ver(-1, -1, -1, None, None)));
        assert_eq!(parse_ver("1.*.3"), None);
        assert_eq!(parse_ver("1.2.*-rc"), None);
    }

    #[test]
    fn precedence() {
        // semver.org's example order
        let ordered = ["0.0.1", "0.1.0", "1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-alpha.beta", "1.0.0-beta", "1.0.0-beta.2", "1.0.0-beta.11", "1.0.0-rc.1", "1.0.0", "1.0.1", "1.2.0", "1.10.0", "2.0.0"];
        for pair in ordered.windows(2) {
            assert!(v(pair[0]) < v(pair[1]), "{} < {}", pair[0], pair[1]);
        }
        assert_eq!(v("1.0.0+a"), v("1.0.0+b"), "build metadata is dropped");
        assert_eq!(cmp_pre(Some("rc.01"), Some("rc.1")), Ordering::Equal, "numbers as numbers");
        assert!(v("1.0.0-rc.01") != v("1.0.0-rc.1"), "but they're different versions");
    }

    #[test]
    fn compatible() {
        assert!(v("1.2.0").compatible(&v("1.9.3")));
        assert!(!v("1.9.0").compatible(&v("2.0.0")));
        assert!(v("0.3.1").compatible(&v("0.3.9")));
        assert!(!v("0.3.1").compatible(&v("0.4.0")));
        assert!(v("0.0.3").compatible(&v("0.0.3")));
        assert!(!v("0.0.3").compatible(&v("0.0.4")));
        assert!(!v("0.1.0").compatible(&v("1.1.0")));
        // pre-releases: their own version's, and the releases from it on
        assert!(v("1.5.0-rc.1").compatible(&v("1.5.0-rc.2")) && v("1.5.0-rc.1").compatible(&v("1.5.0")) && v("1.6.0").compatible(&v("1.5.0-rc.1")));
        assert!(!v("1.4.0").compatible(&v("1.5.0-rc.1")) && !v("1.5.0-rc.1").compatible(&v("1.6.0-rc.1")));
        assert_eq!(v("1.2.0").series(), "1.x");
        assert_eq!(v("0.3.1").series(), "0.3.x");
        assert_eq!(v("0.0.3").series(), "0.0.3");
    }

    #[test]
    fn requirements() {
        let cases: [(&str, &[&str], &[&str]); 16] = [
            ("^1.2.3", &["1.2.3", "1.9.0"], &["1.2.2", "2.0.0", "1.3.0-rc.1"]),
            ("1.2.3", &["1.2.3", "1.4.0"], &["2.0.0"]),
            ("^0.2.3", &["0.2.3", "0.2.9"], &["0.3.0", "0.2.2"]),
            ("^0.0.3", &["0.0.3"], &["0.0.4"]),
            ("^1.2", &["1.2.0", "1.9.9"], &["1.1.9", "2.0.0"]),
            ("^0", &["0.0.1", "0.9.0"], &["1.0.0"]),
            ("~1.2.3", &["1.2.3", "1.2.9"], &["1.3.0", "1.2.2"]),
            ("~1.2", &["1.2.0", "1.2.9"], &["1.3.0"]),
            ("~1", &["1.0.0", "1.9.0"], &["2.0.0"]),
            ("=1.2.3", &["1.2.3"], &["1.2.4"]),
            (">1.2", &["1.3.0", "2.0.0"], &["1.2.9"]),
            (">=1.2.0, <2.0.0", &["1.2.0", "1.99.0"], &["2.0.0", "1.1.0"]),
            ("<=1.2", &["1.2.9", "0.1.0"], &["1.3.0"]),
            ("1.2.*", &["1.2.0", "1.2.7"], &["1.3.0"]),
            ("*", &["0.0.1", "9.0.0"], &["1.0.0-rc.1"]),
            (">=1.5.0-rc.1", &["1.5.0-rc.1", "1.5.0-rc.2", "1.5.0", "1.6.0"], &["1.5.0-alpha", "1.6.0-rc.1", "1.4.0"]),
        ];
        for (text, yes, no) in cases {
            let requirement = req(text);
            for ver in yes { assert!(requirement.matches(&v(ver)), "{text} matches {ver}"); }
            for ver in no { assert!(!requirement.matches(&v(ver)), "{text} doesn't match {ver}"); }
        }
        assert!(req("v1.2.3").matches(&v("1.2.3")));
        for bad in ["", ">", "1.2.3.4", "^*", "1.*.3", ">=1.2-rc", "~x.1", "1.2.3-"] {
            assert!(VersionReq::parse(bad).is_err(), "{bad}");
        }
    }
}
