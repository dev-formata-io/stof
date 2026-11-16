#
# Copyright 2025 Formata, Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

# Smoke test for the Python build: install the wheel (maturin), then `python tests/python/smoke.py`.
import os, socket, subprocess, sys, tempfile, time
from pystof import Doc

failed = 0
def check(name, fn):
    global failed
    try:
        fn()
        print('ok  ', name)
    except Exception as error:
        failed += 1
        print('FAIL', name, '-', str(error).split('\n')[0])

def calls():
    doc = Doc()
    doc.parse('x: 20\nfn add(n: int) -> int { self.x + n }')
    assert doc.call('add', [22]) == 42
    assert doc.call_source('fn f(n: int) -> int { self.x * n }', [2]) == 40

def library_functions():
    doc = Doc()
    doc.lib('App', 'total', lambda *args: sum(args))
    doc.lib('App', 'shape', lambda: None)
    doc.parse('fn f() -> int { App.total(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12) }') # more than 9 arguments
    assert doc.call('f') == 78

def library_functions_per_doc():
    # each document keeps its own library functions: two documents can register the same name
    a = Doc(); b = Doc()
    a.lib('App', 'who', lambda: 'a')
    b.lib('App', 'who', lambda: 'b')
    a.parse("fn f() -> str { App.who() }"); b.parse("fn f() -> str { App.who() }")
    assert a.call('f') == 'a' and b.call('f') == 'b', (a.call('f'), b.call('f'))
    # a document without the function doesn't see another document's
    c = Doc(); c.quiet(True); c.parse("fn f() -> str { App.who() }")
    try:
        missing = str(c.call('f'))
    except Exception as error:
        missing = str(error)
    assert missing not in ('a', 'b'), missing

def library_functions_threads():
    # a document's library functions work from any thread, not only the one that registered them
    import threading
    doc = Doc()
    doc.lib('App', 'double', lambda n: n * 2)
    doc.parse("fn f(n: int) -> int { App.double(n) }")
    results, errors, lock = [], [], threading.Lock()
    def work(n):
        try:
            with lock: results.append(doc.call('f', [n]))
        except Exception as error:
            errors.append(str(error))
    threads = [threading.Thread(target=work, args=(n,)) for n in range(8)]
    for thread in threads: thread.start()
    for thread in threads: thread.join()
    assert not errors, errors[0]
    assert sorted(results) == [n * 2 for n in range(8)], results

def shared_across_threads():
    # one document, many threads, no lock of their own: each call runs whole, one at a time
    import threading
    doc = Doc()
    doc.parse("count: 0\nfn slow() -> int { for (const i in 20000) { self.count += 1; } self.count }")
    results, errors = [], []
    def work():
        try:
            results.append(doc.call('slow')); doc.get('count'); doc.set('seen', True)
        except Exception as error:
            errors.append(str(error))
    threads = [threading.Thread(target=work) for _ in range(6)]
    for thread in threads: thread.start()
    for thread in threads: thread.join()
    assert not errors, errors[0]
    assert sorted(results) == [20000 * n for n in range(1, 7)], results

def gil_released():
    # other Python threads run while a call does
    import threading
    ticks, stop = [], threading.Event()
    def tick():
        while not stop.is_set(): ticks.append(1); time.sleep(0.001)
    thread = threading.Thread(target=tick); thread.start()
    doc = Doc(); doc.parse("fn busy() -> int { let t = 0; for (const i in 300000) { t += 1; } t }")
    try:
        before = len(ticks); doc.call('busy'); after = len(ticks)
    finally:
        stop.set(); thread.join()
    assert after - before > 3, (before, after)

def in_process_server():
    # a call's Http::fetch reaches a server running on a thread in this process (it would wait out its timeout if
    # the call held the GIL)
    import http.server, threading
    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200); self.end_headers(); self.wfile.write(b'hi')
        def log_message(self, *args): pass
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        doc = Doc(); doc.allow_http_hosts(['127.0.0.1'])
        doc.parse('fn f() -> str { Http.text(await Http.fetch("http://127.0.0.1:%d/", timeout = 5s)) }' % server.server_address[1])
        start = time.time()
        assert doc.call('f') == 'hi'
        assert time.time() - start < 2
    finally:
        server.shutdown()

def reentry():
    # a library function calling back into the document that called it is an error (never a deadlock), and the
    # document works after; calling another document is fine
    doc = Doc()
    doc.lib('App', 'back', lambda: doc.get('x'))
    doc.parse("x: 1\nfn f() -> str { try { App.back(); 'no error' } catch (e, message) { message } }")
    assert 'Already borrowed' in doc.call('f'), doc.call('f')
    assert doc.get('x') == 1
    other = Doc(); other.parse('y: 42')
    doc.lib('App', 'other', lambda: other.get('y'))
    doc.parse("fn g() -> int { App.other() }")
    assert doc.call('g') == 42

def values():
    doc = Doc()
    doc.parse('fn f() -> set { const s: set = {[1], [2]}; s }\nfn g() -> map { const m = map(); m.insert([1], 2); m }')
    assert len(doc.call('f')) == 2   # a set of lists (Python can't hash them) comes back as a list
    assert list(doc.call('g').values()) == [2]  # an unhashable map key becomes its text

def memory_limit():
    doc = Doc()
    doc.parse('fn probe() { let s = "aaaaaaaaaaaaaaaa"; loop { s = s + s; } }\nfn ok() -> int { 42 }')
    try:
        doc.call('probe')
        raise AssertionError('no error')
    except ValueError as error:
        assert 'memory limit' in str(error), str(error)
    assert doc.call('ok') == 42

def sandbox():
    doc = Doc()
    doc.parse('fn f() -> str { try { fs.read_string("/etc/hostname"); "read" } catch { "blocked" } }')
    assert doc.call('f') == 'blocked'

def network():
    # a server in another process (in_process_server, below, covers one in this process)
    folder = tempfile.mkdtemp()
    with open(os.path.join(folder, 'hi'), 'w') as f: f.write('hi')
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0)); port = s.getsockname()[1]
    server = subprocess.Popen([sys.executable, '-m', 'http.server', str(port), '--bind', '127.0.0.1', '--directory', folder],
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(50):
        try:
            socket.create_connection(('127.0.0.1', port), timeout=0.1).close(); break
        except OSError: time.sleep(0.1)
    try:
        network_checks(port)
    finally:
        server.terminate()

def network_checks(port):
    src = 'fn f() -> str { try { Http.text(await Http.fetch("http://127.0.0.1:%d/hi")) } catch (e, message) { `blocked: ${message}` } }' % port
    doc = Doc(); doc.quiet(); doc.allow_http(); doc.parse(src)
    res = doc.call('f')
    assert res.startswith('blocked') and 'private network' in res, res
    doc = Doc(); doc.allow_http(); doc.allow_private_network(); doc.parse(src)
    assert doc.call('f') == 'hi'
    doc = Doc(); doc.allow_http_hosts(['127.0.0.1']); doc.parse(src)
    assert doc.call('f') == 'hi'
    assert doc.disallow_http_hosts(['127.0.0.1'])
    doc.quiet()
    assert 'not an allowed host' in doc.call('f')
    # a confined sealed part reaches only its own hosts
    doc = Doc(); doc.quiet(); doc.allow_http_hosts(['127.0.0.1']); doc.parse('tool: { %s }' % src)
    doc.seal('tool', hosts=['api.example.com'])
    assert 'not a host this sealed part may reach' in doc.call('tool.f')
    doc = Doc(); doc.allow_http_hosts(['127.0.0.1']); doc.parse('tool: { %s }' % src)
    doc.seal('tool', hosts=['127.0.0.1'])
    assert doc.call('tool.f') == 'hi'

def html():
    doc = Doc()
    doc.parse("#[tag('p')] view: { #[attr] style: 'position:fixed'; #[attr] title: 'x'; }")
    assert doc.string_export('html', 'view') == '<p title="x"></p>'
    doc.allow_html_attributes(['style'])
    assert 'style=' in doc.string_export('html', 'view')

ENGINE = '''
#[type] Engine: { fn price(units: int) -> int { units * 50 } }
fn attack() { parse("fn price(units: int) -> int { 0 }", <Engine>); }
fn price() -> int { <Engine>.price(2) }
'''
SECRET = 'nWGxne/9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A=' # a test key

def sealing():
    doc = Doc(); doc.quiet(); doc.parse(ENGINE)
    digest = doc.seal('<Engine>')
    assert len(digest) == 64 and digest == doc.digest('<Engine>')
    try:
        doc.call('attack'); raise AssertionError('not refused')
    except ValueError as error:
        assert 'sealed' in str(error), str(error)
    assert doc.call('price') == 100
    # signed: verifies, then seals; a changed document fails
    signature = Doc.sign(SECRET, 'stof/seal', digest); public = Doc.public_key(SECRET)
    assert Doc.verify(public, 'stof/seal', digest, signature)
    other = Doc(); other.parse(ENGINE)
    assert other.seal('<Engine>', public_key=public, signature=signature) == digest
    tampered = Doc(); tampered.parse(ENGINE.replace('* 50', '* 5'))
    try:
        tampered.seal('<Engine>', public_key=public, signature=signature); raise AssertionError('not refused')
    except ValueError as error:
        assert 'signed' in str(error), str(error)

def confined():
    doc = Doc(); doc.quiet(); doc.parse('''
        secrets: { key: 'abc' }
        inputs: { city: 'Boston' }
        tools: { weather: { fn city() -> unknown { root.inputs.city } fn key() -> unknown { root.secrets.key } } }
    ''')
    assert doc.sealed('tools.weather') is None
    doc.seal('tools.weather', removable=True, reads=['inputs'])
    assert doc.call('tools.weather.city') == 'Boston'
    assert doc.call('tools.weather.key') is None
    info = doc.sealed('tools.weather')
    assert info['removable'] and info['part'] == 'root.tools.weather' and info['confined'][0]['reads'] == ['inputs'], info
    assert doc.remove('tools.weather')

def untrusted():
    doc = Doc(); doc.quiet(True); doc.parse("secrets: { key: 'abc' }")
    ran = True
    try:
        doc.parse('leak: root.secrets.key', None, 'untrusted')
    except Exception as error:
        ran = False
        assert 'values must be literals' in str(error), str(error)
    assert not ran, 'an expression ran while parsing untrusted source'
    doc.parse("tool: { label: 'Weather'\nfn f(city: str) -> str { 'sunny in ' + city }\nfn lib() -> str { Key.generate() } }", None, 'untrusted')
    doc.seal('tool', removable=True, reads=[], hosts=[], libs=['Str'], time=2000)
    assert doc.call('tool.f', ['Austin']) == 'sunny in Austin'
    try:
        message = str(doc.call('tool.lib'))
    except Exception as error:
        message = str(error)
    assert "Key library isn't one it may use" in message, message
    assert doc.sealed('tool')['confined'][0]['libs'] == ['Str']
    # into an object by path (not only the main root); a path with no object there is an error, not a quiet no-op
    doc.parse('plugins: { weather: {} }')
    doc.parse("fn forecast() -> str { 'sunny' }", 'plugins.weather', 'untrusted')
    assert doc.call('plugins.weather.forecast') == 'sunny'
    try:
        doc.parse("fn f() -> str { 'x' }", 'plugins.missing', 'untrusted'); missing = 'parsed'
    except Exception as error:
        missing = str(error)
    assert "no object at 'plugins.missing'" in missing, missing

def gates():
    doc = Doc(); doc.quiet(True)
    doc.parse('''
        link: { fn request(route: str) -> str { Host.sign('limitr-network/v1/request', route) } }
        tool: {
            api: { request: super.super.link.request }
            fn calls() -> str { self.api.request('/forecast') }
            fn direct() -> str { Host.sign('limitr-network/v1/voucher', 'more spend') }
        }''')
    doc.lib('Host', 'sign', lambda context, message: f'signed {context} {message}', False)
    doc.seal('tool', removable=True, reads=[], libs=['Str'], calls=['link.request'])
    assert doc.call('tool.calls') == 'signed limitr-network/v1/request /forecast'
    try:
        message = str(doc.call('tool.direct'))
    except Exception as error:
        message = str(error)
    assert "Host library isn't one it may use" in message, message
    assert doc.sealed('tool')['confined'][0]['calls'] == ['root.link.request']

def keys():
    secret = Doc.generate_key(); public = Doc.public_key(secret)
    assert secret != Doc.generate_key()
    order = '{"item":"lamp","qty":2}'
    signature = Doc.sign(secret, 'orders/v1', order)
    assert Doc.verify(public, 'orders/v1', order, signature)
    assert Doc.verify(public, 'orders/v1', order.encode(), signature)
    assert not Doc.verify(public, 'orders/v2', order, signature)
    assert Doc.verify('11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=', 'orders/v1', order,
        'Qr/vFkG4ZLv/M1JwvCdDjSb107rdqjXLjbE044YT2U712OHn2obDLuLns6JNo06QBqCe5ENesDH02ZL4uQqVDQ==')
    doc = Doc(); doc.parse("fn check(m: str, k: str, s: str) -> bool { Key::verify(k, 'orders/v1', m, s) }")
    assert doc.call('check', [order, public, signature]) is True

for name, fn in [('calls', calls), ('sealing', sealing), ('confined', confined), ('untrusted', untrusted), ('gates', gates), ('keys', keys), ('library functions', library_functions), ('library functions per doc', library_functions_per_doc), ('library functions across threads', library_functions_threads), ('shared across threads', shared_across_threads), ('GIL released', gil_released), ('in-process server', in_process_server), ('reentry', reentry), ('values', values), ('memory limit', memory_limit),
                 ('sandbox', sandbox), ('network', network), ('html', html)]:
    check(name, fn)
print(f'{failed} failed' if failed else 'all ok')
raise SystemExit(1 if failed else 0)
