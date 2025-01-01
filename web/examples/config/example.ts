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

// Import Stof and create an empty document
import { Stof } from 'jsr:@formata/stof';
const doc = await Stof.create();

// Have to provide the file system "read" function since Stof is sandboxed
doc.insertLibrary('fs', [
    ['read', (path: string): string => Deno.readTextFileSync(path)]
]);

// Import our config file and add our own interface
doc.importString('stof', `
    import 'web/examples/config/config.stof' as Config;

    fn volume(): str {
        return Config.entity.cube.volume();
    }

    fn delay(): ms {
        return Config.delay;
    }
`);
console.log(doc.call('volume', [])); // prints '0.28m' (14cm * 2m * 1)
console.log(doc.call('delay', []));  // prints '2000'  (2s -> ms)
