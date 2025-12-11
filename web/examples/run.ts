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

import { StofDoc, stof } from '../doc.ts';
await StofDoc.initialize();

const doc = stof`
    value: 42

    async fn another_process() -> int {
        self.value
    }

    #[main]
    fn main() {
        //pln('Liftoff:', await self.another_process());
        //pln(await Custom.test('CJ'));

        const response = await Http.fetch('https://restcountries.com/v3.1/region/europe');
        pln(response);
        pln(Http.success(response));
        pln(Http.text(response));
    }
`;

doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));
doc.lib('Std', 'err', (... vars: unknown[]) => console.error(... vars));
doc.lib('Http', 'fetch', async (
    url: string,
    method: string = 'GET',
    body: Uint8Array | null = null,
    headers: Map<string, string> = new Map()): Promise<Map<string, unknown>> => {
    const response = await fetch(url, {
        method,
        body: body ?? undefined,
        headers,
    });
    const result = new Map<string, unknown>();
    result.set('status', response.status);
    result.set('ok', response.ok);
    result.set('headers', new Map(response.headers));
    result.set('content_type', response.headers.get('content-type') ?? response.headers.get('Content-Type') ?? 'text/plain');
    result.set('bytes', await response.bytes());
    return result;
}, true);

doc.lib('Custom', 'test',
    async function(name: string): Promise<string> {
        const func = async (): Promise<string> => 'this is nested';
        return `Hello, ${name}, ${await func()}`;
    }, true);

const res = await doc.run();
console.log(res);

// deno run --allow-all web/examples/run.ts
// Liftoff: 42
// Hello, CJ from JS function