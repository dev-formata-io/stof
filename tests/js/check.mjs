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
const server = http.createServer((req, res) => {
    if (req.url === '/redirect') { res.writeHead(302, { location: `http://localhost:${server.address().port}/` }); res.end(); return; }
    res.end('hi');
}).listen(0, '127.0.0.1');
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
await check('network: disallowHttp removes a host; allowHttp adds hosts', async () => {
    const doc = await StofDoc.new(fetchSrc(local)); doc.allowHttp(['api.example.com']); doc.allowHttp(['127.0.0.1']);
    assert(await doc.call('f') === 'hi', 'added');
    assert(doc.disallowHttp(['127.0.0.1']), 'removed');
    assert((await doc.call('f')).includes('not an allowed host'), 'refused after');
    const any = await StofDoc.new(fetchSrc(local)); any.allowHttp();
    assert(!any.disallowHttp(['127.0.0.1']), 'no list to remove from');
});
await check('network: a confined sealed part reaches only its hosts, on every redirect', async () => {
    const src = (url) => `tool: { ${fetchSrc(url)} }`;
    const port = server.address().port;
    const a = await StofDoc.new(src(local)); a.allowHttp(['127.0.0.1', 'localhost']);
    a.seal('tool', { hosts: ['api.example.com'] });
    assert((await a.call('tool.f')).includes('not a host this sealed part may reach'), 'another host');
    const b = await StofDoc.new(src(local)); b.allowHttp(['127.0.0.1', 'localhost']);
    b.seal('tool', { hosts: ['127.0.0.1'] });
    assert(await b.call('tool.f') === 'hi', 'its host');
    const c = await StofDoc.new(src(`${local}redirect`)); c.allowHttp(['127.0.0.1', 'localhost']);
    c.seal('tool', { hosts: ['127.0.0.1'] });
    assert((await c.call('tool.f')).includes('not a host this sealed part may reach'), `redirected away (port ${port})`);
    const d = await StofDoc.new(src(`${local}redirect`)); d.allowHttp(['127.0.0.1', 'localhost']);
    assert(await d.call('tool.f') === 'hi', 'unsealed: the redirect is followed');
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
    const signature = StofDoc.sign(SECRET, 'stof/seal', digest);
    const publicKey = StofDoc.publicKey(SECRET);
    assert(StofDoc.verify(publicKey, 'stof/seal', digest, signature), 'verify');
    let wrong = ''; try { (await StofDoc.new(ENGINE)).seal('<Engine>', { publicKey, signature: StofDoc.sign(SECRET, 'engine/v1', digest) }); } catch (e) { wrong = String(e?.message ?? e); }
    assert(/signed/.test(wrong), `only under stof/seal: ${wrong}`);
    const other = await StofDoc.new(ENGINE);
    assert(other.seal('<Engine>', { publicKey, signature }) === digest, 'signed seal');
    const tampered = await StofDoc.new(ENGINE.replace('* 50', '* 5'));
    let error = ''; try { tampered.seal('<Engine>', { publicKey, signature }); } catch (e) { error = String(e?.message ?? e); }
    assert(/signed/.test(error), `tampered: ${error}`);
    // the same digest from BSTF (an engine shipped as bytes)
    const loaded = await StofDoc.new(doc.stof.binaryExport('bstf', null), 'bstf');
    assert(loaded.digest('<Engine>') === digest, 'digest after BSTF');
});

await check('seal: confined parts reach only what they read (by path); removable parts can be dropped; sealed()', async () => {
    const doc = await StofDoc.new(`
        secrets: { key: 'abc' }
        inputs: { city: 'Boston' }
        tools: { weather: {
            fn city() -> unknown { root.inputs.city }
            fn key() -> unknown { root.secrets.key }
            fn poke() { root.inputs.city = 'x'; }
        } }
        fn key() -> unknown { root.secrets.key }`);
    doc.quiet();
    assert(doc.sealed('tools.weather') === null, 'not sealed yet');
    doc.seal('tools.weather', { removable: true, reads: ['inputs'], hosts: ['api.weather.example'] });
    assert(await doc.call('tools.weather.city') === 'Boston', 'reads');
    assert(await doc.call('tools.weather.key') === null, 'not elsewhere');
    await rejects(doc.call('tools.weather.poke'), /sealed/);
    assert(await doc.call('key') === 'abc', 'the rest is unconfined');
    assert(doc.remove('inputs') && doc.parse("inputs: { city: 'Denver' }"), 'replaced'); // a new object at the path
    assert(await doc.call('tools.weather.city') === 'Denver', 'reads follow the path');
    const info = doc.sealed('tools.weather');
    assert(info.removable && !info.fields && info.part === 'root.tools.weather', JSON.stringify(info));
    assert(info.confined.length === 1 && info.confined[0].reads[0] === 'inputs' && info.confined[0].hosts[0] === 'api.weather.example', JSON.stringify(info));
    assert(doc.remove('tools.weather'), 'removable');
});
await check('parse: untrusted source is declarations only, then sealed confined', async () => {
    const doc = await StofDoc.new(`secrets: { key: 'abc' }\ntool: {}`);
    doc.quiet();
    let error = ''; try { doc.parse('leak: root.secrets.key', 'stof', 'tool', 'untrusted'); } catch (e) { error = String(e?.message ?? e); }
    assert(error.includes('values must be literals'), error);
    error = ''; try { doc.parse("import './x.json' as self.x;", 'stof', 'tool', 'untrusted'); } catch (e) { error = String(e?.message ?? e); }
    assert(error.includes("can't import"), error);
    doc.parse(`label: 'Weather'\nfn f(city: str) -> str { 'sunny in ' + city }`, 'stof', 'tool', 'untrusted');
    doc.seal('tool', { removable: true, reads: [], hosts: [], libs: ['Str'], time: 2000 });
    assert(await doc.call('tool.f', 'Austin') === 'sunny in Austin', 'its functions run, sealed');
});
await check('seal: confined code calls only the libraries it may (your own included), for at most its time', async () => {
    const doc = await StofDoc.new(`
        tools: { weather: {
            fn split() -> list { 'a/b'.split('/') }
            fn signs() -> str { Host.sign('limitr-network/v1/voucher', 'more spend') }
            fn spin() -> int { let i = 0; while (i < 100000000) i += 1; i }
        } }
        fn signs() -> str { Host.sign('limitr-network/v1/voucher', 'more spend') }`);
    doc.quiet();
    doc.lib('Host', 'sign', (context, message) => `signed ${context} ${message}`, false, ['context', 'message']);
    doc.seal('tools.weather', { removable: true, reads: [], libs: ['Str'], time: 100 });
    assert(JSON.stringify(await doc.call('tools.weather.split')) === '["a","b"]', 'a listed library');
    await rejects(doc.call('tools.weather.signs'), /Host library isn't one it may use/);
    assert((await doc.call('signs')).startsWith('signed'), 'the rest of the document still has it');
    await rejects(doc.call('tools.weather.spin'), /past its time limit/);
    const info = doc.sealed('tools.weather');
    assert(info.confined[0].libs[0] === 'Str' && info.confined[0].time === 100, JSON.stringify(info));
});
await check('seal: gates are the confined part\'s only way to a host library', async () => {
    const doc = await StofDoc.new(`
        link: { fn request(route: str) -> str { Host.sign('limitr-network/v1/request', route) } }
        tool: {
            api: { request: super.super.link.request }
            fn calls() -> str { self.api.request('/forecast') }
            fn direct() -> str { Host.sign('limitr-network/v1/voucher', 'more spend') }
        }`);
    doc.quiet();
    doc.lib('Host', 'sign', (context, message) => `signed ${context} ${message}`, false, ['context', 'message']);
    doc.seal('tool', { removable: true, reads: [], libs: ['Str'], calls: ['link.request'] });
    assert(await doc.call('tool.calls') === 'signed limitr-network/v1/request /forecast', 'through its gate');
    await rejects(doc.call('tool.direct'), /Host library isn't one it may use/);
    assert(doc.sealed('tool').confined[0].calls[0] === 'root.link.request', JSON.stringify(doc.sealed('tool')));
    let error = ''; try { doc.seal('tool', { calls: ['tool.calls'] }); } catch (e) { error = String(e?.message ?? e); }
    assert(error.includes('outside the part'), error);
});
await check('keys: sign on the host, verify in the document, and the other way around', async () => {
    const secret = StofDoc.generateKey();
    assert(secret !== StofDoc.generateKey() && secret.length === 44, 'random');
    const publicKey = StofDoc.publicKey(secret);
    const order = JSON.stringify({ item: 'lamp', qty: 2 });
    const signature = StofDoc.sign(secret, 'orders/v1', order);
    assert(StofDoc.verify(publicKey, 'orders/v1', order, signature), 'verifies');
    assert(StofDoc.verify(publicKey, 'orders/v1', new TextEncoder().encode(order), signature), 'bytes');
    assert(!StofDoc.verify(publicKey, 'orders/v2', order, signature), 'another context');
    assert(!StofDoc.verify(publicKey, 'orders/v1', order + ' ', signature), 'another message');
    // the fixed signature (made natively) verifies here too
    assert(StofDoc.verify('11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=', 'orders/v1', '{"item":"lamp","qty":2}',
        'Qr/vFkG4ZLv/M1JwvCdDjSb107rdqjXLjbE044YT2U712OHn2obDLuLns6JNo06QBqCe5ENesDH02ZL4uQqVDQ=='), 'the same scheme everywhere');
    const doc = await StofDoc.new(`
        fn check(m: str, k: str, s: str) -> bool { Key::verify(k, 'orders/v1', m, s) }
        fn sign(secret: str, m: str) -> str { Key::sign(secret, 'orders/v1', m) }`);
    assert(await doc.call('check', order, publicKey, signature) === true, 'in a document');
    assert(await doc.call('check', '{}', publicKey, signature) === false, 'in a document, changed');
    const made = await doc.call('sign', secret, order);
    assert(StofDoc.verify(publicKey, 'orders/v1', order, made), "the document's signature on the host");
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
