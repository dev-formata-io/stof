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

import { StofDoc, stof } from '../doc.ts';
await StofDoc.initialize();

const doc = stof`
    value: 42

    async fn another_process() -> int {
        self.value
    }

    #[main]
    fn main() {
        pln('Liftoff:', await self.another_process());
        pln(Custom.test('CJ'));
    }
`;

doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));
doc.lib('Std', 'err', (... vars: unknown[]) => console.error(... vars));
doc.lib('Custom', 'test', (name: string): string => `Hello, ${name} from JS function`);

doc.run();

// deno run --allow-all web/examples/run.ts
// Liftoff: 42
// Hello, CJ from JS function