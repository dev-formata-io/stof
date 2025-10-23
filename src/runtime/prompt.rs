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

use arcstr::ArcStr;
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::runtime::Val;


#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash, PartialEq, Eq)]
pub struct Prompt {
    pub text: ArcStr,
    pub tag: Option<ArcStr>,
    pub prompts: Vector<Self>,
}
impl From<&str> for Prompt {
    fn from(value: &str) -> Self {
        Self {
            text: value.into(),
            ..Default::default()
        }
    }
}
impl From<String> for Prompt {
    fn from(value: String) -> Self {
        Self {
            text: value.into(),
            ..Default::default()
        }
    }
}
impl From<ArcStr> for Prompt {
    fn from(value: ArcStr) -> Self {
        Self {
            text: value,
            ..Default::default()
        }
    }
}
impl From<&ArcStr> for Prompt {
    fn from(value: &ArcStr) -> Self {
        Self {
            text: value.clone(),
            ..Default::default()
        }
    }
}
impl From<(&str, &str)> for Prompt {
    fn from(value: (&str, &str)) -> Self {
        Self {
            text: value.0.into(),
            tag: Some(value.1.into()),
            ..Default::default()
        }
    }
}
impl From<(String, String)> for Prompt {
    fn from(value: (String, String)) -> Self {
        Self {
            text: value.0.into(),
            tag: Some(value.1.into()),
            ..Default::default()
        }
    }
}
impl From<(&ArcStr, &ArcStr)> for Prompt {
    fn from(value: (&ArcStr, &ArcStr)) -> Self {
        Self {
            text: value.0.clone(),
            tag: Some(value.1.clone()),
            ..Default::default()
        }
    }
}
impl From<&Val> for Prompt {
    fn from(value: &Val) -> Self {
        value.to_string().into()
    }
}
impl ToString for Prompt {
    fn to_string(&self) -> String {
        let mut out = self.text.to_string();
        for prompt in &self.prompts {
            out.push_str(&prompt.to_string());
        }
        if let Some(tag) = &self.tag {
            if tag.len() > 0 {
                return format!("<{tag}>{out}</{tag}>");
            }
        }
        out
    }
}
impl Prompt {
    pub fn string_val(&self) -> Val {
        Val::Str(self.to_string().into())
    }

    pub fn set_tag(&mut self, tag: Option<ArcStr>) {
        self.tag = tag;
    }
}
