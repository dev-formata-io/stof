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

use std::{ops::DerefMut, sync::Arc, time::Duration};
use parking_lot::RwLock;
use crate::model::SId;


/// Wake reference.
/// Will wake when value is true.
pub type WakeRef = Arc<RwLock<bool>>;


/// Helper function for waking a process.
/// Returns true if the process was not yet awake.
///
/// Note: if a waker has an "at" duration, this will override that
/// and wake the process anyways.
///
/// If using this from a tokio runtime (or similar), you'll have to
/// have the runtime in a separate thread so that it can make progress
/// independantly from this runtime loop.
pub fn wake(wref: &WakeRef) -> bool {
    let mut val = wref.write();
    let val = val.deref_mut();
    let res = !*val;
    *val = true;
    res
}


/// Helper function for resetting a wake value.
/// Returns true if the reset took place.
pub fn reset(wref: &WakeRef) -> bool {
    let mut val = wref.write();
    let val = val.deref_mut();
    let res = *val;
    *val = false;
    res
}


#[derive(Debug, Default)]
/// Waker.
pub struct Waker {
    pub pid: SId,
    pub at: Option<Duration>,
    pub with: WakeRef,
}
impl Waker {
    /// Woken up?
    pub fn woken(&self, dur: &Duration) -> bool {
        if let Some(at) = &self.at {
            if at <= dur { return true; }
        }
        *self.with.read()
    }

    /// Create a function that when called, will notify the waker and wake it up.
    pub fn waker(&self) -> Box<dyn Fn()> {
        let clone = self.with.clone();
        let func = move || {
            let mut val = clone.write();
            let val = val.deref_mut();
            *val = true;
        };
        Box::new(func)
    }
}
