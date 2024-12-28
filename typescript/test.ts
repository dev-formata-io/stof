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

import { StofDoc } from '../deno_pkg/main.ts';
//import { STOF } from './stof.ts';
//await STOF.initialize();

const doc = new StofDoc('mydocument', '', 'json');
doc.insertLibFunc('console', 'log', (...params: unknown[])=>console.log(...params));
doc.insertLibFunc('fs', 'read', (path: string): string => {
    const decoder = new TextDecoder('utf-8');
    const data = Deno.readFileSync(path);
    return decoder.decode(data);
});
doc.createLibs();

// Import a JSON file
doc.fileImport('json', 'src/json/tests/example.json', 'json', 'Import');

// Add an interface and call it
doc.stringImport('stof', `
    #[main]
    fn main(): str {
        return stringify(Import, 'toml');
    }
`, '');
const res = doc.callFunc('main', []);
console.log(res);
