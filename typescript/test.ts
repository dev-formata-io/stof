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

import { STOF } from './stof.ts';
await STOF.initialize();


// Testing whether a library can mutate a document
const doc = STOF.parse(`
    body: {
        message: 'hello, world'
    }

    #[main]
    fn main(): str {
        let res = IO.fetch('https://stof.dev', stringify(self.body, 'json'));
        return res;
    }
`);
doc.insertLibFunc('console', 'log', (...params: unknown[])=>console.log(...params));
doc.insertLibFunc('IO', 'fetch', (url: string, body: string): string => {
    return JSON.stringify({
        url,
        response: JSON.parse(body)
    });
});
doc.createLibs();

const res = doc.callFunc('main', []);
console.log(res);
