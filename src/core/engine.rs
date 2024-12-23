//
// Copyright 2024 Formata, Inc. All rights reserved.
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

use std::sync::Arc;
use tokio::{runtime::{Builder, Runtime}, sync::Mutex, task};
use crate::SFunc;
use super::SDoc;


/// Stof Engine.
pub struct SEngine {
    pub doc: Arc<Mutex<SDoc>>,
    pub runtime: Arc<Runtime>,
}
impl Default for SEngine {
    fn default() -> Self {
        Self::new(SDoc::default())
    }
}
impl SEngine {
    /// Create a new engine.
    pub fn new(doc: SDoc) -> Self {
        Self {
            doc: Arc::new(Mutex::new(doc)),
            runtime: Arc::new(Builder::new_current_thread().build().unwrap()),
        }
    }

    /// Run the main functions on a node or within this document.
    /// Main functions are denoted with a #[main] attribute in the text format.
    pub fn run(&self) {
        self.runtime.block_on(async {
            let functions;
            {
                let doc = self.doc.lock().await;
                functions = SFunc::all_funcs(&doc.graph);
            }
            
            let local = task::LocalSet::new();
            let doc_clone = self.doc.clone();

            local.run_until(async move {
                let mut pidset = Vec::new();
                let mut handles = tokio::task::JoinSet::new();
                for func in functions {
                    if let Some(attr_val) = func.attributes.get("main") {
                        let pid;
                        {
                            let mut doc = doc_clone.lock().await;
                            pid = doc.processes.spawn();
                            pidset.push(pid.clone());
                        }
                        let clone = doc_clone.clone();
                        if attr_val.is_empty() {
                            handles.spawn_local(async move {
                                return func.engine_call(&pid, clone, vec![], true).await;
                            });
                        } else {
                            let attr_val_clone = attr_val.clone();
                            handles.spawn_local(async move {
                                return func.engine_call(&pid, clone, vec![attr_val_clone], true).await;
                            });
                        }
                    }
                }

                while let Some(_res) = handles.join_next().await {
                    // Nothing to do here...
                }

                // Now kill the created processes
                let mut doc = doc_clone.lock().await;
                for pid in pidset {
                    doc.processes.kill(&pid);
                }
            }).await;
        });
    }
}


#[cfg(test)]
mod tests {
    use crate::{SDoc, SEngine};

    #[test]
    fn run_engine() {
        let stof = r#"
        
        #[main]
        fn main() {
            for (j in 10) pln(j);
        }

        #[main]
        fn hello() {
            pln('hello, world');
        }


        /// Nth number of the fibonacci sequence
        fn fib(n: int): int {
            let last2 = 0;
            let last1 = 1;
            let new = 0;
            for (i in n - 1) {
                new = last1 + last2;
                last2 = last1;
                last1 = new;
            }
            return new;
        }

        #[main]
        fn runfib() {
            pln('FIB: ', self.fib(40));
        }

        "#;
        let doc = SDoc::src(&stof, "stof").unwrap();
        let engine = SEngine::new(doc);
        engine.run();
    }
}
