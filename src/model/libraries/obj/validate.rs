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
use imbl::{vector, Vector};
use crate::{model::{obj::SCHEMAFY, Field, Func, Graph, NodeRef, SId}, runtime::{instruction::Instruction, instructions::{call::{FuncCall, NamedArg}, Base}, Type, Val, ValRef, Variable}};


/// Validation instructions.
pub fn validation(graph: &mut Graph, schema: &NodeRef, target: &NodeRef, field: String, validate: Val, schema_val: ValRef<Val>, target_val: Option<ValRef<Val>>, remove_invalid: bool, remove_undefined: bool) -> Vec<Vector<Arc<dyn Instruction>>> {
    let mut validation_instructions = Vec::new();
    match validate {
        Val::Void |
        Val::Null => {
            // If schema_val and target_val are objects, do a schemafy on them
            if let Some(schema) = schema_val.read().try_obj() {
                if let Some(target_val) = &target_val {
                    if let Some(target) = target_val.read().try_obj() {
                        let instructions = vector![
                            Arc::new(Base::Literal(Val::Obj(schema))) as Arc<dyn Instruction>,
                            Arc::new(Base::Literal(Val::Obj(target))) as Arc<dyn Instruction>,
                            Arc::new(Base::Literal(Val::Bool(remove_invalid))) as Arc<dyn Instruction>,
                            Arc::new(Base::Literal(Val::Bool(remove_undefined))) as Arc<dyn Instruction>,
                            SCHEMAFY.clone(),
                        ];
                        validation_instructions.push(instructions);
                    }
                }
            }
        },
        Val::Obj(additional_schema) => {
            // Do a schemafy with an additional schema object for this field alone
            if let Some(schema_field_ref) = Field::field(graph, &additional_schema, &field) {
                let mut schema_attr_val = None;
                let mut schema_field_val = None;
                if let Some(field) = graph.get_stof_data::<Field>(&schema_field_ref) {
                    if let Some(attr) = field.attributes.get("schema") {
                        schema_attr_val = Some(attr.clone());
                        schema_field_val = Some(field.value.val.duplicate(false));
                    }
                }
                if let Some(schema_val) = schema_field_val {
                    if let Some(validate) = schema_attr_val {
                        let mut field_instructions = validation(graph, &additional_schema, &target, field, validate, schema_val, target_val, remove_invalid, remove_undefined);
                        validation_instructions.append(&mut field_instructions);
                    }
                }
            }
        },
        Val::Fn(func_ref) => {
            // Use a validation function
            // (schema: obj, target: obj, field: str, schema_val: unknown, target_val: unknown) -> bool
            if let Some(func) = graph.get_stof_data::<Func>(&func_ref) {
                let mut schema_param_name = "schema".to_string();
                let mut target_param_name = "target".to_string();
                let mut field_param_name = "field".to_string();
                let mut schema_val_name = "schema_val".to_string();
                let mut target_val_name = "target_val".to_string();

                let mut seen_schema = false;
                let mut seen_target = false;
                let mut seen_field = false;
                let mut seen_schema_val = false;
                let mut seen_target_val = false;

                for param in &func.params {
                    match &param.param_type {
                        Type::Obj(_) => {
                            let param_name = param.name.as_ref();
                            if !param_name.contains("val") {
                                if param_name == "schema" || (param_name != "target" && !seen_schema) {
                                    schema_param_name = param_name.to_string();
                                    seen_schema = true;
                                } else if param.name.as_ref() == "target" || seen_schema {
                                    target_param_name = param_name.to_string();
                                    seen_target = true;
                                }
                            } else if param_name.contains("schema") || (!param_name.contains("target") && !seen_schema_val) {
                                schema_val_name = param_name.to_string();
                                seen_schema_val = true;
                            } else if param_name.contains("target") || !seen_target_val {
                                target_val_name = param_name.to_string();
                                seen_target_val = true;
                            }
                        },
                        Type::Str => {
                            let param_name = param.name.as_ref();
                            if !param_name.contains("val") && !seen_field {
                                seen_field = true;
                                field_param_name = param_name.to_string();
                            } else if param_name.contains("schema") || (!param_name.contains("target") && !seen_schema_val) {
                                schema_val_name = param_name.to_string();
                                seen_schema_val = true;
                            } else if param_name.contains("target") || !seen_target_val {
                                target_val_name = param_name.to_string();
                                seen_target_val = true;
                            }
                        },
                        _ => {
                            let param_name = param.name.as_ref();
                            if param_name.contains("schema") || (!param_name.contains("target") && !seen_schema_val) {
                                schema_val_name = param_name.to_string();
                                seen_schema_val = true;
                            } else if param_name.contains("target") || !seen_target_val {
                                target_val_name = param_name.to_string();
                                seen_target_val = true;
                            }
                        }
                    }
                }

                let mut args: Vector<Arc<dyn Instruction>> = vector![];
                if seen_schema {
                    args.push_back(Arc::new(NamedArg {
                        name: SId::from(&schema_param_name),
                        ins: Arc::new(Base::Literal(Val::Obj(schema.clone())))
                    }));
                }
                if seen_target {
                    args.push_back(Arc::new(NamedArg {
                        name: SId::from(&target_param_name),
                        ins: Arc::new(Base::Literal(Val::Obj(target.clone())))
                    }));
                }
                if seen_field {
                    args.push_back(Arc::new(NamedArg {
                        name: SId::from(&field_param_name),
                        ins: Arc::new(Base::Literal(Val::Str(field.into())))
                    }));
                }
                if seen_schema_val {
                    args.push_back(Arc::new(NamedArg {
                        name: SId::from(&schema_val_name),
                        ins: Arc::new(Base::Variable(Variable::refval(schema_val)))
                    }));
                }
                if seen_target_val {
                    if let Some(target_val) = target_val {
                        args.push_back(Arc::new(NamedArg {
                            name: SId::from(&target_val_name),
                            ins: Arc::new(Base::Variable(Variable::refval(target_val)))
                        }));
                    } else {
                        args.push_back(Arc::new(NamedArg {
                            name: SId::from(&target_val_name),
                            ins: Arc::new(Base::Variable(Variable::val(Val::Null)))
                        }));
                    }
                }

                let mut instructions = vector![
                    Arc::new(FuncCall {
                        func: Some(func_ref),
                        search: None,
                        stack: false,
                        as_ref: false,
                        cnull: false,
                        args,
                        oself: None,
                    }) as Arc<dyn Instruction>
                ];
                if func.return_type.empty() {
                    instructions.push_back(Arc::new(Base::Literal(Val::Bool(true))));
                }
                validation_instructions.push(instructions);
            }
        },
        Val::Set(list) => {
            for validate in list {
                let validate = validate.read().clone();
                validation_instructions.append(&mut validation(graph, schema, target, field.clone(), validate, schema_val.clone(), target_val.clone(), remove_invalid, remove_undefined));
            }
        },
        Val::Tup(list) |
        Val::List(list) => {
            // Use a pipeline of validation objects
            for validate in list {
                let validate = validate.read().clone();
                validation_instructions.append(&mut validation(graph, schema, target, field.clone(), validate, schema_val.clone(), target_val.clone(), remove_invalid, remove_undefined));
            }
        },
        _ => {
            // Just add a false literal to the stack (not valid)
            validation_instructions.push(vector![
                Arc::new(Base::Literal(Val::Bool(false))) as Arc<dyn Instruction>
            ]);
        }
    }
    validation_instructions
}
