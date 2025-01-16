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

import { Stof } from "../../doc.ts";

// There are other ways of doing this with Stof, but this way simulates
// getting this data from external data sources.
const record = await Deno.readTextFile("web/examples/patient/record.json");
const stof = await Deno.readTextFile("web/examples/patient/interface.stof");

// We parse both strings into a Stof document
const doc = await Stof.parse('json', record);
doc.importString('stof', stof, 'Interface');

// Then we can use our interfaces, along with using the original data
const official = doc.get('Interface.officialName');
const usual = doc.get('Interface.usualName');
const id = doc.get('id');

console.log(official);
console.log(usual);
console.log(id);