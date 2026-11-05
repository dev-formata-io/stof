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

//! Memory budget: how much a call may grow the heap before it fails with `Error::MemoryLimit`.
//!
//! Running out of memory can't be recovered from (in wasm it traps the whole engine, natively it aborts the
//! process), so a document must be stopped before it gets there. Two checks do it:
//! - **Growth during a call:** `StofAlloc` counts heap bytes per thread, and before every instruction the runtime
//!   adds up how much the call has grown the heap (one thread-local read). Each step of a call is measured on the
//!   thread that runs it, so async calls that resume on another thread are still counted right. This catches
//!   many small allocations (a list that keeps growing) and repeated doubling (`s = s + s`).
//! - **One big step:** operations that can make a value many times larger than their inputs in one instruction
//!   (`'a' * n`, `replace`, `split`, `join`, blob to list, `parse`) estimate the size first and refuse when it's
//!   over the limit. This works without `StofAlloc`.
//!
//! The JavaScript and Python builds install `StofAlloc`. A Rust host can install it too (it wraps the system
//! allocator); without it, only the one-big-step checks apply. Memory a call keeps (Ex. fields it sets) counts
//! against that call only; the next call starts from zero.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use crate::runtime::Error;


/// Default memory budget for a call: 256 MiB.
pub const DEFAULT_MAX_MEMORY: usize = 256 * 1024 * 1024;

/// Heap cost of one list, set, or map element (a value cell, its value, and the list's share), for estimates.
/// Measured at about 136 bytes for a number pushed to a list and 155 for a short string from `split`.
pub const ELEMENT_COST: usize = 160;

/// Estimated growth of `parse` per byte of source (documents are bigger in memory than as text).
pub const PARSE_COST: usize = 8;

thread_local! {
    /// Bytes allocated minus bytes freed on this thread (wrapping; only counted when `StofAlloc` is the global
    /// allocator). Thread-local, so counting is a plain add (no atomics) and one call's steps aren't mixed with
    /// other threads' work. `const` and `Cell`: no lazy init, no destructor, nothing allocated.
    static NET: Cell<usize> = const { Cell::new(0) };
}

#[inline]
fn add(bytes: usize) {
    let _ = NET.try_with(|net| net.set(net.get().wrapping_add(bytes)));
}

#[inline]
fn sub(bytes: usize) {
    let _ = NET.try_with(|net| net.set(net.get().wrapping_sub(bytes)));
}


/// The system allocator, counting live heap bytes for the memory budget.
///
/// ```ignore
/// #[global_allocator]
/// static ALLOC: stof::runtime::memory::StofAlloc = stof::runtime::memory::StofAlloc;
/// ```
pub struct StofAlloc;
unsafe impl GlobalAlloc for StofAlloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() { add(layout.size()); }
        ptr
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() { add(layout.size()); }
        ptr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        sub(layout.size());
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            if new_size >= layout.size() {
                add(new_size - layout.size());
            } else {
                sub(layout.size() - new_size);
            }
        }
        new_ptr
    }
}


#[inline]
/// This thread's heap counter (a mark to measure from with `since`). Always 0 when `StofAlloc` isn't the global
/// allocator.
pub fn mark() -> usize {
    NET.try_with(|net| net.get()).unwrap_or(0)
}

#[inline]
/// How much this thread's heap has grown (negative: shrunk) since `mark`.
pub fn since(mark: usize) -> isize {
    self::mark().wrapping_sub(mark) as isize
}


#[inline]
/// Refuse an operation that would make a value of about `bytes` bytes when that's over the limit (None for no
/// limit). For operations that can grow a value many times over in one step.
pub fn check_size(max: Option<usize>, bytes: usize) -> Result<(), Error> {
    match max {
        Some(max) if bytes > max => Err(Error::MemoryLimit(max)),
        _ => Ok(()),
    }
}


#[inline]
/// `check_size` for `count` elements of `each` bytes (saturating, so huge counts are refused, not wrapped).
pub fn check_elements(max: Option<usize>, count: usize, each: usize) -> Result<(), Error> {
    check_size(max, count.saturating_mul(each))
}


/// Megabytes, for messages.
pub fn mb(bytes: usize) -> String {
    let mb = bytes as f64 / (1024. * 1024.);
    if mb >= 10. || mb.fract() == 0. { format!("{} MB", mb.round()) } else { format!("{mb:.1} MB") }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks() {
        assert!(check_size(None, usize::MAX).is_ok());
        assert!(check_size(Some(10), 10).is_ok());
        assert!(check_size(Some(10), 11).is_err());
        assert!(check_elements(Some(100), usize::MAX, 64).is_err());
        assert_eq!(since(mark()), 0);
        assert_eq!(mb(DEFAULT_MAX_MEMORY), "256 MB");
        assert_eq!(mb(1536 * 1024), "1.5 MB");
    }
}
