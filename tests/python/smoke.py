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
    # a server in another process: a Stof call holds the GIL while it waits on a request
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

for name, fn in [('calls', calls), ('sealing', sealing), ('confined', confined), ('untrusted', untrusted), ('gates', gates), ('keys', keys), ('library functions', library_functions), ('values', values), ('memory limit', memory_limit),
                 ('sandbox', sandbox), ('network', network), ('html', html)]:
    check(name, fn)
print(f'{failed} failed' if failed else 'all ok')
raise SystemExit(1 if failed else 0)
