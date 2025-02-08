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

use colored::Colorize;
use serde::{Deserialize, Serialize};
use crate::{SData, SDataRef, SDoc, SFunc, SGraph, SVal};


/// Stof error types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    TypeError,
    CallError,
    ParseError,

    // Standard Library Errors
    StdLibError(String),
    ArrayLibError(String),
    BlobLibError(String),
    DataLibError(String),
    BoolLibError(String),
    FuncLibError(String),
    MapLibError(String),
    NumberLibError(String),
    ObjectLibError(String),
    SetLibError(String),
    StringLibError(String),
    TupleLibError(String),

    // Special system libraries
    FileSystemLibError(String),
    TimeLibError(String),

    FormatError(String),
    ThrownError(String), // error for when users call "throw"
    ValueError(String),
    Custom(String),
}
impl ErrorType {
    /// To string.
    pub fn to_string(&self) -> String {
        match self {
            Self::Custom(error) => error.clone(),
            Self::ThrownError(error) => error.clone(),
            _ => format!("{:?}", self),
        }
    }
}


/// Stof runtime error.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SError {
    pub pid: String,
    pub error_type: ErrorType,
    pub message: String,
    pub call_stack: Vec<SDataRef>,
}
impl SError {
    pub fn new(pid: &str, doc: &SDoc, etype: ErrorType, message: &str) -> Self {
        let mut call_stack = Vec::new();
        if let Some(process) = doc.processes.get(pid) {
            call_stack = process.call_stack.clone();
        }
        Self {
            pid: pid.to_owned(),
            error_type: etype,
            message: message.to_owned(),
            call_stack,
        }
    }

    /// Type error.
    pub fn type_error(pid: &str, doc: &SDoc, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::TypeError, message)
    }

    /// Parse error.
    pub fn parse(pid: &str, doc: &SDoc, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::ParseError, message)
    }

    /// Call error.
    pub fn call(pid: &str, doc: &SDoc, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::CallError, message)
    }

    /// Standard library error.
    pub fn std(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::StdLibError(func.to_owned()), message)
    }

    /// Array library error.
    pub fn array(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::ArrayLibError(func.to_owned()), message)
    }

    /// Blob library error.
    pub fn blob(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::BlobLibError(func.to_owned()), message)
    }

    /// Data library error.
    pub fn data(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::DataLibError(func.to_owned()), message)
    }

    /// Bool library error.
    pub fn bool(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::BoolLibError(func.to_owned()), message)
    }

    /// Function library error.
    pub fn func(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::FuncLibError(func.to_owned()), message)
    }

    /// Map library error.
    pub fn map(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::MapLibError(func.to_owned()), message)
    }

    /// Set library error.
    pub fn set(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::SetLibError(func.to_owned()), message)
    }

    /// String library error.
    pub fn string(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::StringLibError(func.to_owned()), message)
    }

    /// Number library error.
    pub fn num(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::NumberLibError(func.to_owned()), message)
    }

    /// Tuple library error.
    pub fn tup(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::TupleLibError(func.to_owned()), message)
    }

    /// Object library error.
    pub fn obj(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::ObjectLibError(func.to_owned()), message)
    }

    /// FileSystem library error.
    pub fn filesys(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::FileSystemLibError(func.to_owned()), message)
    }

    /// Time library error.
    pub fn time(pid: &str, doc: &SDoc, func: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::TimeLibError(func.to_owned()), message)
    }

    /// User thrown error.
    pub fn thrown(pid: &str, doc: &SDoc, etype: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::ThrownError(etype.to_owned()), message)
    }

    /// Value error.
    pub fn val(pid: &str, doc: &SDoc, op: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::ValueError(op.to_owned()), message)
    }

    /// Custom error.
    pub fn custom(pid: &str, doc: &SDoc, error: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::Custom(error.to_owned()), message)
    }

    /// Format error.
    pub fn fmt(pid: &str, doc: &SDoc, format: &str, message: &str) -> Self {
        Self::new(pid, doc, ErrorType::FormatError(format.to_owned()), message)
    }

    /// Empty format error.
    pub fn empty_fmt(format: &str, message: &str) -> Self {
        Self {
            pid: "main".to_string(),
            error_type: ErrorType::FormatError(format.to_string()),
            message: message.to_owned(),
            call_stack: Default::default(),
        }
    }

    /// (type: str, message: str, stack: Vec<fn>)
    pub fn to_value(&self) -> SVal {
        let call_stack = self.call_stack.iter().map(|dref| SVal::FnPtr(dref.clone())).collect::<Vec<SVal>>();
        SVal::Tuple(vec![SVal::String(self.error_type.to_string()), SVal::String(self.message.to_owned()), SVal::Array(call_stack)])
    }

    /// To error string.
    pub fn to_string(&self, graph: &SGraph) -> String {
        let mut res = String::default();
        for dref in &self.call_stack {
            if let Some(func) = SData::get::<SFunc>(&graph, dref) {
                let func_nodes = dref.nodes(&graph);
                let func_path;
                if func_nodes.len() > 0 {
                    func_path = func_nodes.first().unwrap().path(&graph);
                } else {
                    func_path = String::from("<unknown>");
                }

                res.push_str(&format!("\t{} {} {} {} ...\n", "unwind".red(), func.name.blue(), "@".dimmed(), func_path.italic().bright_cyan()));
            }
        }
        res.push_str(&format!("\t{}: {}", self.error_type.to_string().purple(), self.message.dimmed()));
        res
    }
}
