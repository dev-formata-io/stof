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


/// Error.
/// TODO.
pub enum Error {
    Custom(String),

    /// Attempting to declare a variable that already exists.
    DeclareExisting,
    /// Attempting to declare a variable with an invalid name.
    DeclareInvalidName,
    /// Attempting to declare a variable with an invalid type.
    DeclareInvalidType(Box<Self>),
}
impl Error {
    /// Custom error string.
    pub fn custom(message: impl ToString) -> Self {
        Self::Custom(message.to_string())
    }
}
