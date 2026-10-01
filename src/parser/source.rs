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

//! Source locations for error messages.
//!
//! Parsers work on `&str` slices of one document, so a slice's address says where it is in the source.
//! `document()` registers the text it parses (address range, file, line starts) for the duration of the
//! parse, and any parser (most are context-free) can turn a position into file:line:col with `locate`.
//! Nothing here is stored in documents (BSTF) and nothing runs at runtime.

use std::cell::RefCell;
use arcstr::ArcStr;


#[derive(Debug, Clone, PartialEq, Eq)]
/// A source location (1-based line and column).
pub struct SrcLoc {
    pub file: Option<ArcStr>,
    pub line: u32,
    pub col: u32,
}
impl SrcLoc {
    /// "file:line:col", or "line:col" without a file.
    pub fn display(&self) -> String {
        match &self.file {
            Some(file) => format!("{file}:{}:{}", self.line, self.col),
            None => format!("{}:{}", self.line, self.col),
        }
    }
}


struct Source {
    start: usize,
    len: usize,
    file: Option<ArcStr>,
    /// Byte offset of each line start.
    line_starts: Vec<usize>,
    /// Emit statement location markers (debug profiles)?
    markers: bool,
}

thread_local! {
    /// Documents being parsed on this thread (innermost last; imports nest).
    static SOURCES: RefCell<Vec<Source>> = RefCell::new(Vec::new());
}


thread_local! {
    /// Current nesting depth of recursive parsers (expressions, blocks, object values).
    static NESTING: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Max nesting of expressions/blocks/objects while parsing. Recursive descent uses the stack for each level,
/// so very deep (usually malicious or broken) input would overflow it and abort (wasm: trap the engine).
/// Real documents stay far below it (the Stof test suite peaks at 9, Limitr at 13). Release builds use ~2.5KB
/// of stack per level (128 levels fit in 512KB; wasm has 1MB); unoptimized builds use 10x more.
/// With the `stacker` feature (native builds), deep levels continue on a fresh stack segment instead of
/// overflowing, so this is the same for every build type and thread size (128 is also serde_json's limit).
pub const MAX_NESTING: usize = 128;

/// Run a recursive step with enough stack: with the `stacker` feature (all native feature sets), when less than
/// 128KB of stack is left, continue on a new 2MB segment (heap allocated). Otherwise (wasm) just run it: release
/// wasm uses ~2.5KB per level, so MAX_NESTING levels fit easily in its 1MB stack.
#[inline(always)]
pub fn grow<R>(f: impl FnOnce() -> R) -> R {
    #[cfg(feature = "stacker")]
    { stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, f) }
    #[cfg(not(feature = "stacker"))]
    { f() }
}

/// One level of parser nesting, released when dropped. None when the input is nested too deeply.
pub struct NestingGuard;
impl NestingGuard {
    pub fn enter() -> Option<Self> {
        NESTING.with(|depth| {
            if depth.get() >= MAX_NESTING { return None; }
            depth.set(depth.get() + 1);
            Some(NestingGuard)
        })
    }
}
impl Drop for NestingGuard {
    fn drop(&mut self) {
        NESTING.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}


/// Registration of a source text, removed when dropped.
pub struct SourceGuard;
impl Drop for SourceGuard {
    fn drop(&mut self) {
        SOURCES.with(|sources| { sources.borrow_mut().pop(); });
    }
}


/// Register a source text while it is being parsed.
/// The line index is one pass over the bytes (negligible next to parsing).
pub fn push_source(text: &str, file: Option<ArcStr>, markers: bool) -> SourceGuard {
    let mut line_starts = Vec::with_capacity(text.len() / 32 + 1);
    line_starts.push(0);
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' { line_starts.push(index + 1); }
    }
    let source = Source { start: text.as_ptr() as usize, len: text.len(), file, line_starts, markers };
    SOURCES.with(|sources| sources.borrow_mut().push(source));
    SourceGuard
}


#[inline]
/// Emit statement location markers for the document being parsed?
pub fn markers_enabled() -> bool {
    SOURCES.with(|sources| sources.borrow().last().map(|source| source.markers).unwrap_or(false))
}


/// Location of a position (a slice starting there) in a registered source.
/// Columns are byte based (exact for ASCII).
pub fn locate(position: &str) -> Option<SrcLoc> {
    locate_addr(position.as_ptr() as usize)
}

/// Location of a position address (see `locate`).
pub fn locate_addr(addr: usize) -> Option<SrcLoc> {
    SOURCES.with(|sources| {
        let sources = sources.borrow();
        for source in sources.iter().rev() {
            if addr >= source.start && addr <= source.start + source.len {
                let offset = addr - source.start;
                let line = source.line_starts.partition_point(|start| *start <= offset); // 1-based
                let line_start = source.line_starts[line - 1];
                return Some(SrcLoc { file: source.file.clone(), line: line as u32, col: (offset - line_start + 1) as u32 });
            }
        }
        None
    })
}


/// Line and column (1-based, columns in chars) of a byte offset in a text, with the text of that line.
pub fn line_col(text: &str, offset: usize) -> (usize, usize, &str) {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) { offset -= 1; }
    let before = &text[..offset];
    let line_start = before.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let col = text[line_start..offset].chars().count() + 1;
    let line_end = text[line_start..].find('\n').map(|index| line_start + index).unwrap_or(text.len());
    (line, col, text[line_start..line_end].trim_end_matches('\r'))
}


/// A code frame: the source line with a caret under the column.
/// ```text
///    |
///  7 |     let x = 5
///    |              ^
/// ```
pub fn code_frame(line: usize, col: usize, line_text: &str) -> String {
    // long lines (Ex. minified input): show a window around the column
    const WINDOW: usize = 60;
    let chars: Vec<char> = line_text.chars().collect();
    let (line_text, col) = if chars.len() > WINDOW * 2 {
        let start = col.saturating_sub(1).saturating_sub(WINDOW).min(chars.len());
        let end = (col + WINDOW).min(chars.len());
        let mut window: String = chars[start..end].iter().collect();
        let mut col = col - start;
        if start > 0 { window.insert_str(0, "..."); col += 3; }
        if end < chars.len() { window.push_str("..."); }
        (window, col)
    } else {
        (line_text.to_string(), col)
    };
    let line_text = line_text.as_str();
    let number = line.to_string();
    let pad = " ".repeat(number.len());
    // keep tabs so the caret lines up with the source
    let mut caret_pad = String::new();
    for ch in line_text.chars().take(col.saturating_sub(1)) {
        caret_pad.push(if ch == '\t' { '\t' } else { ' ' });
    }
    format!(" {pad} |\n {number} | {line_text}\n {pad} | {caret_pad}^")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_positions() {
        let text = "a: 1\nfn main() {\n    let x = 5\n}";
        let _guard = push_source(text, Some("main.stof".into()), false);
        let loc = locate(&text[text.find("let").unwrap()..]).unwrap();
        assert_eq!((loc.line, loc.col), (3, 5));
        assert_eq!(loc.display(), "main.stof:3:5");
        assert_eq!(locate(&text[0..]).map(|l| (l.line, l.col)), Some((1, 1)));
        assert_eq!(locate("elsewhere"), None);

        let (line, col, src) = line_col(text, text.find("= 5").unwrap());
        assert_eq!((line, col, src), (3, 11, "    let x = 5"));
        assert_eq!(code_frame(line, col, src), "   |\n 3 |     let x = 5\n   |           ^");
    }
}
