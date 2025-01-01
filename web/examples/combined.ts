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

import { Stof } from 'jsr:@formata/stof';

// Create a doc.
const doc = await Stof.create();

// Import some JSON.
doc.importString('json', JSON.stringify({
    colors: [
        {
            name: 'red',
            value: '#f00'
        },
        {
            name: 'green',
            value: '#0f0'
        }
    ]
}));

// Import TOML.
doc.importString('toml', `
[[colors]]
name = "blue"
value = "#00f"
`);

// Export YAML.
console.log(doc.exportString('yaml'));

// Add some Stof and run.
doc.importString('stof', `
    #[main]
    fn main() {
        for (color in self.colors) {
            console.log(color.toString());
        }
    }
`);
doc.run();
