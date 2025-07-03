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
use arcstr::ArcStr;
use imbl::Vector;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, ConsumeStack, POP_SYMBOL_SCOPE, PUSH_SYMBOL_SCOPE, TRUTHY}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// While statement.
pub struct WhileIns {
    // custom continue and break tags (cannot be generic!)
    pub continue_tag: ArcStr,
    pub break_tag: ArcStr,

    // the real stuff
    pub test: Arc<dyn Instruction>,
    pub ins: Vector<Arc<dyn Instruction>>,

    // added control flow stuff for flexibility
    pub declare: Option<Arc<dyn Instruction>>,
    pub inc: Option<Arc<dyn Instruction>>,
}
#[typetag::serde(name = "WhileIns")]
impl Instruction for WhileIns {
    fn exec(&self, instructions: &mut Instructions, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        instructions.push(PUSH_SYMBOL_SCOPE.clone());

        if let Some(declare) = &self.declare {
            instructions.push(declare.clone());
        }
        
        let top_tag: ArcStr = nanoid!(10).into();
        let end_tag: ArcStr = nanoid!(10).into();

        instructions.push(Arc::new(Base::Tag(top_tag.clone())));
        {
            // Create another symbol scope just for this iteration
            instructions.push(PUSH_SYMBOL_SCOPE.clone());

            // Test if the value is truthy, go to end_tag if not
            instructions.push(self.test.clone());
            instructions.push(TRUTHY.clone());
            instructions.push(Arc::new(Base::CtrlForwardToIfNotTruthy(end_tag.clone(), ConsumeStack::Consume)));
            
            // Do the thing
            instructions.append(&self.ins);

            // Continue statements will go to here
            instructions.push(Arc::new(Base::Tag(self.continue_tag.clone())));

            // If we have an inc expr, do that now before we start the loop again
            if let Some(inc) = &self.inc {
                instructions.push(inc.clone());
            }

            // Get rid of the iteration symbol table and go back to the top
            instructions.push(POP_SYMBOL_SCOPE.clone());
            instructions.push(Arc::new(Base::CtrlBackTo(top_tag)));
        }

        // Break statements will go here, as well as our jump if not truthy
        instructions.push(Arc::new(Base::Tag(self.break_tag.clone())));
        instructions.push(Arc::new(Base::Tag(end_tag)));
        instructions.push(POP_SYMBOL_SCOPE.clone());
        instructions.push(POP_SYMBOL_SCOPE.clone());
        Ok(())
    }
}
