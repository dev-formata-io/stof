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

// Checks for the JavaScript build (the wasm engine and doc.ts): build web/dist, then `node tests/js/check.mjs`.
// Pass a dist folder to check another build (Ex. node tests/js/check.mjs path/to/dist).
import http from 'node:http';
import path from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.argv[2] ?? path.join(here, '../../web/dist'));
const { StofDoc, initStof } = await import(pathToFileURL(path.join(dist, 'doc.js')).href);
await initStof();

let failed = 0;
async function check(name, fn) {
    try {
        await fn();
        console.log('ok  ', name);
    } catch (error) {
        failed++;
        console.log('FAIL', name, '-', String(error?.message ?? error).split('\n')[0]);
    }
}
function assert(cond, message) { if (!cond) throw new Error(message); }
async function rejects(promise, pattern) {
    try { await promise; } catch (error) {
        const message = String(error?.message ?? error);
        assert(pattern.test(message), `wrong error: ${message.split('\n')[0]}`);
        return;
    }
    throw new Error('expected an error');
}
const doubling = 'fn probe() { let s = "aaaaaaaaaaaaaaaa"; loop { s = s + s; } }';

// calls, library functions, and calling back into the document
await check('call, callSource, run', async () => {
    const doc = await StofDoc.new('x: 20\nfn add(n: int) -> int { self.x + n }\n#[main] fn main() -> int { 1 }');
    assert(await doc.call('add', 22) === 42, 'call');
    assert(await doc.callSource('fn f(n: int) -> int { self.x * n }', [2]) === 40, 'callSource');
    assert(!doc.stringify('json').includes('__source'), 'callSource left its source behind');
    assert((await doc.run()).includes('main'), 'run report');
    assert(doc.sync_call('add', 1) === 21, 'sync_call');
});
await check('async library function (a host promise, resolved on a timer)', async () => {
    const doc = await StofDoc.new('fn f() -> int { (await App.later(41)) + 1 }');
    doc.lib('App', 'later', (n) => new Promise((resolve) => setTimeout(() => resolve(n), 30)), true);
    assert(await doc.call('f') === 42, 'value');
});
await check('a library function calls back with await doc.call', async () => {
    const doc = await StofDoc.new('x: 2\nfn inner() -> int { self.x * 10 }\nfn outer() -> int { (await App.back()) + 1 }');
    doc.lib('App', 'back', async () => await doc.call('inner'), true); // before its first await, like before
    assert(await doc.call('outer') === 21, 'value');
    doc.lib('App', 'back', async () => { await new Promise((r) => setTimeout(r, 5)); return await doc.callSource('fn f() -> int { self.x }'); }, true);
    assert(await doc.call('outer') === 3, 'after an await');
});
await check('a synchronous call back during a step is refused (busy), not a crash', async () => {
    const doc = await StofDoc.new('fn inner() -> int { 1 }\nfn outer() -> str { App.back() }');
    doc.lib('App', 'back', () => { try { doc.sync_call('inner'); return 'no'; } catch (e) { return String(e).includes('busy') ? 'busy' : String(e); } });
    assert(await doc.call('outer') === 'busy', 'busy');
    assert(await doc.call('inner') === 1, 'still works');
});
await check('many documents, many concurrent calls', async () => {
    const docs = await Promise.all([...Array(8)].map((_, i) => StofDoc.new(`fn f(n: int) -> int { let t = 0; for (const i in n) t += ${i}; t }`)));
    const results = await Promise.all(docs.flatMap((doc, i) => [doc.call('f', 1000), doc.call('f', 10)]));
    assert(results.every((r, k) => r === Math.floor(k / 2) * (k % 2 ? 10 : 1000)), JSON.stringify(results));
});

// memory limit
await check('memory limit: a runaway async call fails, the document still works', async () => {
    const doc = await StofDoc.new(doubling + '\nfn ok() -> int { 42 }');
    await rejects(doc.call('probe'), /memory limit/);
    assert(await doc.call('ok') === 42, 'doc still works');
});
await check('memory limit: in #[init] while parsing, and on a synchronous call', async () => {
    // an #[init] that fails is reported as a warning, not a parse error: the parse finishes, the engine doesn't crash
    const init = await StofDoc.new('obj: { #[init] fn boot() { let s = "aaaaaaaaaaaaaaaa"; loop { s = s + s; } } }\nfn ok() -> int { 42 }');
    assert(await init.call('ok') === 42, 'parse with a runaway #[init]');
    const doc = await StofDoc.new(doubling);
    let sync = '';
    try { doc.sync_call('probe'); } catch (e) { sync = String(e?.message ?? e); }
    assert(/memory limit/.test(sync), `sync: ${sync.split('\n')[0]}`);
});
await check('memory limit: one big step ("a" * n) and setMaxMemory', async () => {
    const doc = await StofDoc.new('fn big(n: int) -> int { ("abc" * n).len() }');
    await rejects(doc.call('big', 1e12), /memory limit/);
    doc.setMaxMemory(1024 * 1024);
    await rejects(doc.call('big', 1e6), /memory limit/);
    doc.setMaxMemory(null);
    assert(await doc.call('big', 1e6) === 3e6, 'no limit');
});

// network
const server = http.createServer((req, res) => res.end('hi')).listen(0, '127.0.0.1');
await new Promise((resolve) => server.once('listening', resolve));
const local = `http://127.0.0.1:${server.address().port}/`;
const fetchSrc = (url) => `fn f() -> str { try { const res = await Http.fetch('${url}'); return Http.text(res); } catch (e, message) { return \`blocked: \${message}\`; } }`;
await check('network: any host but the private network', async () => {
    for (const url of [local, 'http://localhost:9/', 'http://169.254.169.254/computeMetadata/v1/', 'http://[::1]:9/', 'http://2130706433:9/', 'http://metadata.google.internal/']) {
        const doc = await StofDoc.new(fetchSrc(url));
        doc.allowHttp();
        const res = await doc.call('f');
        assert(res.startsWith('blocked') && res.includes('private network'), `${url}: ${res}`);
    }
});
await check('network: allowPrivateNetwork, or a host list, reaches it', async () => {
    const a = await StofDoc.new(fetchSrc(local)); a.allowHttp(); a.allowPrivateNetwork();
    assert(await a.call('f') === 'hi', 'allowPrivateNetwork');
    const b = await StofDoc.new(fetchSrc(local)); b.allowHttp(['127.0.0.1']);
    assert(await b.call('f') === 'hi', 'host list');
    const c = await StofDoc.new(fetchSrc(local)); c.allowHttp(['api.example.com']);
    assert((await c.call('f')).includes('not an allowed host'), 'other host');
});
server.close();

// html
await check('html: style and action only when allowed, URL schemes', async () => {
    const doc = await StofDoc.new(`
        #[tag('div')] view: {
            #[tag('a')] ok: { #[attr] href: 'https://stof.dev'; }
            #[tag('a')] js: { #[attr] href: 'javascript:alert(1)'; }
            #[tag('a')] file: { #[attr] href: 'file:///etc/passwd'; }
            #[tag('p')] styled: { #[attr] style: 'position:fixed'; }
        }`);
    const html = doc.stringify('html', 'view');
    assert(html.includes('href="https://stof.dev"') && !html.includes('alert') && !html.includes('file:') && !html.includes('style'), html);
    doc.allowHtmlAttributes(['style']);
    assert(doc.stringify('html', 'view').includes('style="position:fixed"'), 'allowed style');
});

// sealing
const ENGINE = `#[type] Engine: { fn price(units: int) -> int { units * 50 } }
fn attack() { parse("fn price(units: int) -> int { 0 }", <Engine>); }
fn price() -> int { <Engine>.price(2) }`;
const SECRET = 'nWGxne/9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A='; // a test key
await check('seal: functions can be called but not changed; digest and signatures', async () => {
    const doc = await StofDoc.new(ENGINE);
    doc.quiet();
    const digest = doc.seal('<Engine>');
    assert(digest.length === 64 && digest === doc.digest('<Engine>'), 'digest');
    await rejects(doc.call('attack'), /sealed/);
    assert(await doc.call('price') === 100, 'still the sealed price');
    const signature = StofDoc.sign(digest, SECRET);
    const publicKey = StofDoc.publicKey(SECRET);
    assert(StofDoc.verify(digest, publicKey, signature), 'verify');
    const other = await StofDoc.new(ENGINE);
    assert(other.seal('<Engine>', { publicKey, signature }) === digest, 'signed seal');
    const tampered = await StofDoc.new(ENGINE.replace('* 50', '* 5'));
    let error = ''; try { tampered.seal('<Engine>', { publicKey, signature }); } catch (e) { error = String(e?.message ?? e); }
    assert(/signed/.test(error), `tampered: ${error}`);
    // the same digest from BSTF (an engine shipped as bytes)
    const loaded = await StofDoc.new(doc.stof.binaryExport('bstf', null), 'bstf');
    assert(loaded.digest('<Engine>') === digest, 'digest after BSTF');
});

// last: a crash (no memory limit, so the engine runs out of memory and traps)
await check('crash: the call fails, the process lives, the document says it crashed, others work', async () => {
    const doc = await StofDoc.new(doubling);
    doc.setMaxMemory(null);
    await rejects(doc.call('probe'), /crashed/);
    assert(doc.crashed, 'doc.crashed');
    await rejects(doc.call('probe'), /crashed/);
    let sync = ''; try { doc.get('x'); } catch (e) { sync = String(e?.message ?? e); }
    assert(/crashed/.test(sync), 'sync methods say so too');
    doc.dispose();
    const other = await StofDoc.new('fn f() -> int { 42 }');
    assert(await other.call('f') === 42, 'another document');
});

console.log(failed ? `${failed} failed` : 'all ok');
process.exit(failed ? 1 : 0);
