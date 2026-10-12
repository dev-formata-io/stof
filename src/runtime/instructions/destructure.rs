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

use std::sync::Arc;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::{model::{Field, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::{nullcheck::NullcheckIns, Base, POP_STACK}, proc::ProcEnv, Error, Type, Val, ValRef, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// What a destructuring declaration takes apart.
pub enum Pattern {
    /// A variable ("_" ignores the value).
    Name(ArcStr),
    /// Fields of an object, or string keys of a map: (field, target, default).
    /// `{ a, b: other, c = 1, d: { e } }`
    Fields(Vec<(ArcStr, Pattern, Option<Arc<dyn Instruction>>)>),
    /// Items of a list or tuple, by position: (target, default).
    /// `[first, second]`, `(key, value)`
    Items(Vec<(Pattern, Option<Arc<dyn Instruction>>)>),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Destructuring declaration: takes the value on the stack apart into variables.
/// A missing field or item is null (or its default, if it has one); a value of the wrong kind is an error.
pub struct DestructureIns {
    pub pattern: Pattern,
    pub is_const: bool,
}
#[typetag::serde(name = "DestructureIns")]
impl Instruction for DestructureIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let Some(var) = env.stack.pop() else { return Err(Error::StackError); };
        let value = var.get();
        let mut instructions = Instructions::default();
        match &self.pattern {
            Pattern::Name(name) => {
                self.bind(&mut instructions, Some(var), &Pattern::Name(name.clone()), &None);
            },
            Pattern::Fields(fields) => {
                for (key, target, default) in fields {
                    let part = match &value {
                        Val::Obj(obj) => Field::field(graph, obj, key)
                            .and_then(|field| graph.get_stof_data::<Field>(&field))
                            .map(|field| field.value.stack_var(false)),
                        Val::Map(map) => map.get(&ValRef::new(Val::Str(key.clone()))).map(|value| Variable::refval(value.duplicate(false))),
                        other => return Err(cannot_destructure(other, graph, "fields", "an object or map")),
                    };
                    self.bind(&mut instructions, part, target, default);
                }
            },
            Pattern::Items(items) => {
                let (Val::List(values) | Val::Tup(values)) = &value else {
                    return Err(cannot_destructure(&value, graph, "items", "a list or tuple"));
                };
                for (index, (target, default)) in items.iter().enumerate() {
                    let part = values.get(index).map(|value| Variable::refval(value.duplicate(false)));
                    self.bind(&mut instructions, part, target, default);
                }
            },
        }
        Ok(Some(instructions))
    }
}
impl DestructureIns {
    /// Instructions that declare a target with a part (null if missing, then its default).
    fn bind(&self, instructions: &mut Instructions, part: Option<Variable>, target: &Pattern, default: &Option<Arc<dyn Instruction>>) {
        let part: Arc<dyn Instruction> = Arc::new(Base::Variable(part.unwrap_or_else(|| Variable::val(Val::Null))));
        match default {
            Some(default) => instructions.push(Arc::new(NullcheckIns { ins: part, ifnull: default.clone() })),
            None => instructions.push(part),
        }
        match target {
            Pattern::Name(name) if name == "_" => instructions.push(POP_STACK.clone()),
            Pattern::Name(name) if self.is_const => instructions.push(Arc::new(Base::DeclareConstVar(name.clone(), Type::Void))),
            Pattern::Name(name) => instructions.push(Arc::new(Base::DeclareVar(name.clone(), Type::Void))),
            pattern => instructions.push(Arc::new(DestructureIns { pattern: pattern.clone(), is_const: self.is_const })),
        }
    }
}

fn cannot_destructure(value: &Val, graph: &Graph, what: &str, expected: &str) -> Error {
    Error::Custom(format!("cannot destructure {what} from a {} (expected {expected})", value.spec_type(graph).rt_type_of(graph)).into())
}
