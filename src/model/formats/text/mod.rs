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

pub mod md;
use crate::{model::{Field, Format, Graph, NodeRef}, runtime::{Error, Val, Variable}};


#[derive(Debug)]
pub struct TextFormat;
impl Format for TextFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["text".into(), "txt".into()]
    }
    fn content_type(&self) -> String {
        "text/plain".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let mut parse_node = graph.ensure_main_root();
        if let Some(nd) = node {
            parse_node = nd;
        }
        if let Some(field_ref) = Field::direct_field(&graph, &parse_node, "text") {
            let mut fvar = None;
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                fvar = Some(field.value.clone());
            }
            if let Some(mut fvar) = fvar {
                fvar.set(&Variable::val(Val::Str(src.into())), graph, None)?;
                if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                    field.value = fvar;
                }
            }
        } else {
            graph.insert_stof_data(&parse_node, "text", Box::new(Field::new(Variable::val(Val::Str(src.into())), None)), None);
        }
        Ok(())
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default TEXT export");
        }
        if let Some(field_ref) = Field::direct_field(&graph, &exp_node, "text") {
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                Ok(field.value.val.read().print(graph))
            } else {
                Ok("".into())
            }
        } else {
            Ok("".into())
        }
    }
}


#[derive(Debug)]
pub struct MdFormat;
impl Format for MdFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["md".into(), "markdown".into()]
    }
    fn content_type(&self) -> String {
        "text/markdown".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let mut parse_node = graph.ensure_main_root();
        if let Some(nd) = node {
            parse_node = nd;
        }
        if let Some(field_ref) = Field::direct_field(&graph, &parse_node, "md") {
            let mut fvar = None;
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                fvar = Some(field.value.clone());
            }
            if let Some(mut fvar) = fvar {
                fvar.set(&Variable::val(Val::Str(src.into())), graph, None)?;
                if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                    field.value = fvar;
                }
            }
        } else {
            graph.insert_stof_data(&parse_node, "md", Box::new(Field::new(Variable::val(Val::Str(src.into())), None)), None);
        }
        Ok(())
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default MARKDOWN export");
        }
        if let Some(field_ref) = Field::direct_field(&graph, &exp_node, "md") {
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                Ok(field.value.val.read().print(graph))
            } else {
                Ok("".into())
            }
        } else {
            Ok("".into())
        }
    }
}
