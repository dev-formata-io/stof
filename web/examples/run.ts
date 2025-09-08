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

import { Doc } from '../doc.ts';

const doc = await Doc.new();

// add console libs
doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));
doc.lib('Std', 'err', (... vars: unknown[]) => console.trace(... vars));

// custom
doc.lib('Custom', 'test', (name: string): string => {
    return `Hello, ${name} from function`;
});

doc.parse(`
    value: 42

    async fn another_process() -> int {
        self.value
    }

    #[main]
    fn main() {
        let res = await self.another_process();
        pln('We have liftoff: ', res);

        let message = await async Custom.test('CJ');
        pln(message);
    }
`);

doc.run();

// deno run --allow-all web/examples/run.ts
// 42