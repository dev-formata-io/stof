//
// Copyright 2025 Formata, Inc. All rights reserved.
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

import { stofAsync } from '../doc.ts';

const doc = await stofAsync`
    fn hello(name: str) {
        pln('Hello, ' + name);
    }

    #[main]
    fn main() {
        await Gated.callback('World!');
        pln('----');
        await Gated.callback('Stof!');
    }
`;
doc.lib('Std', 'pln', (...args: unknown[])=>console.log(...args));
doc.lib('Gated', 'callback', async (name: string) => {
    await doc.call('hello', name); // this is a feat
});
doc.sync_call('hello', 'dude');

await doc.run();