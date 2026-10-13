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

//! Obj.run: running an object's #[run] steps.
//!
//! A step is a field or function with a #[run] attribute. An object's steps come from its type (and the
//! type's types) plus the object itself, merged by name:
//! - A step's name is its field or function name, or #[run({'name': 'x'})].
//! - A step with the same name as a type's step replaces it, so each step runs once. A null field
//!   (#[run] check: null) removes the type's step.
//! - Steps run in definition order: the type's steps first, then the object's new steps. A replaced step
//!   keeps its place.
//! - A number moves a step, like a z-index for every step together: #[run(-1)] runs before the unnumbered
//!   steps (0), #[run(1)] after. A replacing step without a number keeps the number of the step it replaces.
//!
//! The object's own #[run({..})] map (or the run call) can change this:
//! - 'prototype': 'first' (default, the type's steps first), 'last' (the object's new steps first),
//!   'none' (only the object's own steps), or 'only' (only the type's steps, run on this object).
//! - 'steps': a list of step names to run, in that order (only those steps run).

use std::{ops::Deref, sync::Arc};
use imbl::{vector, Vector};
use rustc_hash::FxHashMap;
use crate::{model::{DataRef, Field, Func, Graph, NodeRef, Prototype}, runtime::{Error, Val, ValRef, Variable, instruction::{Instruction, Instructions}, instructions::{Base, call::FuncCall, empty::EmptyIns}}};
use super::RUN;


/// Which of an object's steps run, and where the type's steps go.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PrototypeMode {
    First,
    Last,
    None,
    Only,
}
impl PrototypeMode {
    fn from_str(mode: &str) -> Result<Self, Error> {
        match mode {
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            "none" => Ok(Self::None),
            "only" => Ok(Self::Only),
            _ => Err(Error::Custom(format!("unknown run prototype mode '{mode}' (use 'first', 'last', 'none', or 'only')").into())),
        }
    }
}


/// One #[run] step.
#[derive(Clone)]
struct RunStep {
    /// Explicit order, if given (unnumbered steps are 0).
    order: Option<f64>,
    /// Arguments for function steps (#[run({'args': [..]})]).
    args: Option<ValRef<Val>>,
    /// What runs.
    action: RunAction,
    /// Defined on a type (prototype) rather than the object itself.
    from_prototype: bool,
}

#[derive(Clone)]
enum RunAction {
    /// A field's value: an object to run, a function to call, or a collection of those (a pipeline).
    Value(Val),
    /// A function with #[run].
    Func(DataRef),
}


/// Instructions that run an object's steps.
/// Context is the object functions run on (self), when it isn't the object itself.
/// Mode and steps come from the run call; when not given, from the object's #[run({..})] attribute.
pub(super) fn run_instructions(graph: &mut Graph, obj: &NodeRef, context: Option<NodeRef>, mode: Option<Val>, steps: Option<Val>) -> Result<Instructions, Error> {
    let (mode, steps) = run_options(graph, obj, mode, steps)?;

    // Levels that hold steps, most derived first: the object, its type, that type's type, ...
    let mut levels = vec![(obj.clone(), false)];
    for proto in Prototype::prototype_nodes(graph, obj, true) {
        levels.push((proto, true));
    }
    let level_steps: Vec<Vec<(String, RunStep)>> = levels.iter()
        .map(|(node, from_prototype)| level_run_steps(graph, node, *from_prototype))
        .collect();

    // The most derived step with a name wins (runs), keeping a number from a step it replaces.
    // Each name also has the level that introduced it (the most base level defining it).
    let mut winners: FxHashMap<&String, &RunStep> = FxHashMap::default();
    let mut orders: FxHashMap<&String, Option<f64>> = FxHashMap::default();
    let mut introduced: FxHashMap<&String, usize> = FxHashMap::default();
    for (level, steps) in level_steps.iter().enumerate() {
        for (name, step) in steps {
            winners.entry(name).or_insert(step);
            let order = orders.entry(name).or_insert(None);
            if order.is_none() { *order = step.order; }
            introduced.insert(name, level);
        }
    }

    // Which steps this run includes
    let included = |name: &String| -> bool {
        match mode {
            PrototypeMode::None => !winners[name].from_prototype, // defined on the object itself
            PrototypeMode::Only => introduced[name] > 0,          // defined by a type (as this object has it)
            _ => true,
        }
    };

    // Plan: the named steps in the order asked for, or every included step in definition order, then by number
    let mut plan: Vec<&String> = Vec::new();
    if let Some(names) = &steps {
        for name in names {
            let Some((name, _)) = winners.get_key_value(name) else {
                return Err(Error::Custom(format!("run: no step named '{name}' on this object").into()));
            };
            plan.push(name);
        }
    } else {
        // Definition order: each step goes where it was first defined (a replaced step keeps its place).
        // The type's steps come first (base type first), or with 'last', the object's new steps come first.
        let mut level_order: Vec<usize> = (0..level_steps.len()).collect();
        if mode != PrototypeMode::Last { level_order.reverse(); }
        for level in level_order {
            for (name, _) in &level_steps[level] {
                if introduced[name] == level && included(name) { plan.push(name); }
            }
        }
        // Stable: steps with the same number stay in definition order
        plan.sort_by(|a, b| orders[a].unwrap_or(0.).total_cmp(&orders[b].unwrap_or(0.)));
    }
    let plan: Vec<&RunStep> = plan.into_iter().map(|name| winners[name]).collect();

    let self_obj = context.clone().unwrap_or(obj.clone());
    let mut instructions = Instructions::default();
    for step in plan {
        // Functions from a type run on this object (self); so does everything when a context is given
        let mut oself = None;
        if context.is_some() || step.from_prototype {
            oself = Some(Arc::new(Base::Literal(Val::Obj(self_obj.clone()))) as Arc<dyn Instruction>);
        }
        let arguments = step_arguments(&step.args);
        match &step.action {
            RunAction::Func(dref) => {
                instructions.push(call_func(dref.clone(), arguments, oself));
            },
            RunAction::Value(value) => {
                match value {
                    Val::List(items) => {
                        for item in items { push_value_step(&mut instructions, item.read().deref(), &arguments, &oself); }
                    },
                    Val::Tup(items) => {
                        for item in items { push_value_step(&mut instructions, item.read().deref(), &arguments, &oself); }
                    },
                    Val::Set(items) => {
                        for item in items { push_value_step(&mut instructions, item.read().deref(), &arguments, &oself); }
                    },
                    Val::Map(map) => {
                        for item in map.values() { push_value_step(&mut instructions, item.read().deref(), &arguments, &oself); }
                    },
                    value => {
                        push_value_step(&mut instructions, value, &arguments, &oself);
                    },
                }
            },
        }
    }
    Ok(instructions)
}


/// Prototype mode and step names, from the run call or else the object's #[run({..})] attribute.
fn run_options(graph: &Graph, obj: &NodeRef, mode: Option<Val>, steps: Option<Val>) -> Result<(PrototypeMode, Option<Vec<String>>), Error> {
    let mut mode = mode.filter(|val| !val.null());
    let mut steps = steps.filter(|val| !val.null());
    if let Some(node) = obj.node(graph) {
        if let Some(Val::Map(map)) = node.attributes.get("run") {
            if mode.is_none() {
                mode = map.get(&ValRef::new(Val::Str("prototype".into()))).map(|val| val.read().clone());
            }
            if steps.is_none() {
                steps = map.get(&ValRef::new(Val::Str("steps".into()))).map(|val| val.read().clone());
            }
        }
    }

    let prototype_mode = match mode {
        None => PrototypeMode::First,
        Some(Val::Str(mode)) => PrototypeMode::from_str(mode.as_str())?,
        Some(_) => return Err(Error::Custom("run prototype mode must be a string ('first', 'last', 'none', or 'only')".into())),
    };
    let step_names = match steps {
        None => None,
        Some(Val::List(names)) => {
            let mut step_names = Vec::new();
            for name in names {
                match name.read().deref() {
                    Val::Str(name) => step_names.push(name.to_string()),
                    _ => return Err(Error::Custom("run steps must be a list of step names (strings)".into())),
                }
            }
            Some(step_names)
        },
        Some(_) => return Err(Error::Custom("run steps must be a list of step names (strings)".into())),
    };
    Ok((prototype_mode, step_names))
}


/// The #[run] steps defined directly on one object, in definition order.
fn level_run_steps(graph: &mut Graph, node: &NodeRef, from_prototype: bool) -> Vec<(String, RunStep)> {
    Field::fields(graph, node); // creates fields for child objects that don't have one yet
    let mut steps = Vec::new();
    let Some(object) = node.node(graph) else { return steps; };
    for (data_name, dref) in &object.data {
        let (attr, action) = if let Some(field) = graph.get_stof_data::<Field>(dref) {
            match field.attributes.get("run") {
                Some(attr) => (attr, RunAction::Value(field.value.val.read().clone())),
                None => continue,
            }
        } else if let Some(func) = graph.get_stof_data::<Func>(dref) {
            match func.attributes.get("run") {
                Some(attr) => (attr, RunAction::Func(dref.clone())),
                None => continue,
            }
        } else {
            continue;
        };

        let mut name = data_name.clone();
        let mut order = None;
        let mut args = None;
        match attr {
            Val::Num(num) => order = Some(num.float(None)),
            Val::Map(map) => {
                if let Some(val) = map.get(&ValRef::new(Val::Str("order".into()))) {
                    if let Val::Num(num) = val.read().deref() { order = Some(num.float(None)); }
                }
                if let Some(val) = map.get(&ValRef::new(Val::Str("args".into()))) {
                    args = Some(val.clone());
                }
                if let Some(val) = map.get(&ValRef::new(Val::Str("name".into()))) {
                    if let Val::Str(step_name) = val.read().deref() { name = step_name.to_string(); }
                }
            },
            _ => {}
        }
        steps.push((name, RunStep { order, args, action, from_prototype }));
    }
    steps
}


/// Argument instructions for a function step.
fn step_arguments(args: &Option<ValRef<Val>>) -> Vector<Arc<dyn Instruction>> {
    let mut arguments: Vector<Arc<dyn Instruction>> = vector![];
    if let Some(args) = args {
        if let Val::List(args) = args.read().deref() {
            for arg in args {
                arguments.push_back(Arc::new(Base::Variable(Variable::refval(arg.clone()))));
            }
        }
    }
    arguments
}


/// Call a function as a step (its return value is discarded).
fn call_func(dref: DataRef, args: Vector<Arc<dyn Instruction>>, oself: Option<Arc<dyn Instruction>>) -> Arc<dyn Instruction> {
    Arc::new(EmptyIns {
        ins: Arc::new(FuncCall { func: Some(dref), search: None, stack: false, as_ref: false, cnull: false, args, oself })
    })
}


/// One value in a step: an object runs (with itself as self), a function is called.
fn push_value_step(instructions: &mut Instructions, value: &Val, args: &Vector<Arc<dyn Instruction>>, oself: &Option<Arc<dyn Instruction>>) {
    match value {
        Val::Obj(other) => {
            instructions.push(Arc::new(Base::Literal(Val::Obj(other.clone()))));
            instructions.push(Arc::new(Base::Literal(Val::Null))); // context: the object itself
            instructions.push(RUN.clone());
        },
        Val::Fn(dref) => {
            instructions.push(call_func(dref.clone(), args.clone(), oself.clone()));
        },
        _ => {}
    }
}

