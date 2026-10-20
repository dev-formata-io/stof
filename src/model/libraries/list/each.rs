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

//! List functions that call a function for each item: map, filter, and any with a predicate.
//! Each call runs in the calling process (one call per item, in order), so a function can do anything a normal call
//! can: an error fails the list function, and an async function's result is its promise (await the list of them).

use std::{ops::Deref, sync::Arc};
use imbl::{vector, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param}, runtime::{Error, Type, Val, ValRef, Variable, instruction::{Instruction, Instructions}, instructions::{Base, call::FuncCall, empty::ValueIns}, proc::ProcEnv}};
use super::LIST_LIB;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Each {
    Map,
    Filter,
    Any,
}

pub fn list_any() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: r#"# List.any(array: list, func?: fn) -> bool
Does this list contain any values? With a function: does it return true (truthy) for any value?
```rust
const array = [1, 2, 3];
assert(array.any());
assert(array.any((x: int): bool => x > 2));
assert_not(array.any((x: int): bool => x > 3));
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "func".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(CallEach { each: Each::Any }));
            Ok(instructions)
        }),
    }
}

pub fn list_map() -> LibFunc {
    each_lib("map", Each::Map, r#"# List.map(array: list, func: fn) -> list
A new list with the result of calling the function for each value, in order.
```rust
const array = [1, 2, 3];
assert_eq(array.map((x: int): int => x * 2), [2, 4, 6]);
```"#)
}

pub fn list_filter() -> LibFunc {
    each_lib("filter", Each::Filter, r#"# List.filter(array: list, func: fn) -> list
A new list with the values the function returns true for (truthy), in order.
```rust
const array = [1, 2, 3, 4];
assert_eq(array.filter((x: int): bool => x % 2 == 0), [2, 4]);
```"#)
}

fn each_lib(name: &str, each: Each, docs: &str) -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: name.into(),
        is_async: false,
        docs: docs.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "func".into(), param_type: Type::Fn, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(CallEach { each }));
            Ok(instructions)
        }),
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Pops a function (or null, for any) and a list, then calls the function for each item.
pub struct CallEach {
    pub each: Each,
}
#[typetag::serde(name = "ListCallEach")]
impl Instruction for CallEach {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let fail = || match self.each {
            Each::Map => Error::ListMap,
            Each::Filter => Error::ListFilter,
            Each::Any => Error::ListAny,
        };
        let Some(func_var) = env.stack.pop() else { return Err(fail()); };
        let Some(list_var) = env.stack.pop() else { return Err(fail()); };
        let items = match list_var.val.read().deref() {
            Val::List(items) => items.clone(),
            _ => return Err(fail()),
        };
        let func = match func_var.val.read().deref() {
            Val::Fn(dref) => dref.clone(),
            Val::Null | Val::Void if self.each == Each::Any => {
                // any() without a function: does the list have values?
                env.stack.push(Variable::val(Val::Bool(!items.is_empty())));
                return Ok(None);
            },
            _ => return Err(fail()),
        };

        let mut instructions = Instructions::default();
        for item in items.iter() {
            instructions.push(Arc::new(ValueIns { ins: Arc::new(FuncCall {
                func: Some(func.clone()),
                search: None,
                stack: false,
                as_ref: false,
                cnull: false,
                args: vector![Arc::new(Base::Variable(Variable::refval(item.clone()))) as Arc<dyn Instruction>],
                oself: None,
            }) }));
        }
        instructions.push(Arc::new(CollectEach { each: self.each, items }));
        Ok(Some(instructions))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Collects the results of a CallEach (one value per item on the stack) into the list function's result.
/// Only created while running, so the items aren't serialized.
pub struct CollectEach {
    pub each: Each,
    #[serde(skip)]
    pub items: Vector<ValRef<Val>>,
}
#[typetag::serde(name = "ListCollectEach")]
impl Instruction for CollectEach {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut results = Vec::with_capacity(self.items.len());
        for _ in 0..self.items.len() {
            match env.stack.pop() {
                Some(var) => results.push(var),
                None => return Err(match self.each { Each::Map => Error::ListMap, Each::Filter => Error::ListFilter, Each::Any => Error::ListAny }),
            }
        }
        results.reverse();
        let res = match self.each {
            Each::Map => Val::List(results.into_iter().map(|var| ValRef::new(var.val.read().deref().clone())).collect()),
            Each::Filter => Val::List(self.items.iter().zip(results).filter(|(_, res)| res.truthy()).map(|(item, _)| item.clone()).collect()),
            Each::Any => Val::Bool(results.iter().any(|res| res.truthy())),
        };
        env.stack.push(Variable::val(res));
        Ok(None)
    }
}
