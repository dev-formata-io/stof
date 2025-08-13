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
use arcstr::{literal, ArcStr};
use imbl::vector;
use markdown::{to_html, to_mdast, ParseOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type, Val, Variable}};


const MD_LIB: ArcStr = literal!("Md");


pub fn insert_md_lib(graph: &mut Graph) {
    // Md.html(md)
    graph.insert_libfunc(LibFunc {
        library: MD_LIB.clone(),
        name: "html".into(),
        is_async: false,
        docs: "# Markdown -> HTML\nParse MD string into an HTML string.".to_string(),
        params: vector![
            Param { name: "md".into(), param_type: Type::Str, default: None, }
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(MdIns::Html));
            Ok(instructions)
        }),
    });

    // Md.json(md)
    graph.insert_libfunc(LibFunc {
        library: MD_LIB.clone(),
        name: "json".into(),
        is_async: false,
        docs: "# Markdown -> JSON\nParse MD string into a JSON string.".to_string(),
        params: vector![
            Param { name: "md".into(), param_type: Type::Str, default: None, }
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(MdIns::Json));
            Ok(instructions)
        }),
    });
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Markdown library instructions.
pub enum MdIns {
    Html,
    Json,
}
#[typetag::serde(name = "MdIns")]
impl Instruction for MdIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Html => {
                if let Some(var) = env.stack.pop() {
                    let md = var.val.read().print(&graph);
                    env.stack.push(Variable::val(Val::Str(to_html(&md).into())));
                }
            },
            Self::Json => {
                if let Some(var) = env.stack.pop() {
                    let md = var.val.read().print(&graph);
                    let options = ParseOptions::default();
                    if let Ok(ast) = to_mdast(&md, &options) {
                        if let Ok(mut value) = serde_json::to_value(&ast) {
                            remove_json_position_data(&mut value);
                            let res = serde_json::to_string(&value).unwrap();
                            env.stack.push(Variable::val(Val::Str(res.into())));
                        }
                    }
                }
            },
        }
        Ok(None)
    }
}


fn remove_json_position_data(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("position"); // remove position data... that sh#$ aint helpful
            if let Some(children) = map.get_mut("children") {
                remove_json_position_data(children);
            }
        },
        Value::Array(vals) => {
            for val in vals {
                if val.is_object() {
                    remove_json_position_data(val);
                }
            }
        },
        _ => {}
    }
}
