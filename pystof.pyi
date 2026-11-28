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

"""Stof for Python (`pip install stof`): data that carries its own logic, run sandboxed in your process."""

from typing import Any, Callable, Dict, List, Literal, Optional, Sequence, Union

Profile = Literal['prod', 'test', 'prod_docs', 'untrusted', 'docs']
Message = Union[str, bytes]

class Doc:
    """
    A Stof document. Safe to share across threads: calls run one at a time (a call from another thread waits its
    turn), and a running call releases the GIL, so other Python threads go on meanwhile.
    """

    def __init__(self) -> None:
        """Create a new document."""

    # Values

    def get(self, path: str, start: Optional[str] = None) -> Any:
        """Get a value by path (Ex. 'config.port'), with an optional starting object (an object ID)."""

    def set(self, path: str, value: Any, start: Optional[str] = None) -> bool:
        """Set a value by path, with an optional starting object (an object ID). False when it can't be set."""

    def remove(self, path: str, shallow: bool = False) -> bool:
        """
        Remove a field, function, or object by path (Ex. 'config.staging'), like drop() in Stof. A field that holds
        an object drops the object too, unless shallow. Returns whether anything was removed.
        """

    # Running

    def call(self, path: str, args: Any = None) -> Any:
        """
        Call a function in the document by path. Arguments are a list (Ex. `doc.call('total', [21])`), a single
        value, or None for no arguments. Raises ValueError with the error when the call fails.
        """

    def call_source(self, src: str, args: Any = None, node: Optional[str] = None) -> Any:
        """
        Call a function from Stof source without adding it to the document (Ex. a one-off script or check). The
        source's first function runs as if it were defined on node (an object path or ID, the root by default), and
        everything the source declares is removed after the call.
        """

    def load(self, node: Optional[str] = None) -> int:
        """
        Run the document's #[load] functions (in node and the objects in it, an object path or ID, or the whole
        document), which bring in its dependencies (Ex. with import), and wait for them. Returns how many ran. Call it
        after parsing; it's safe to call again (imports already there aren't fetched). Imports use the document's
        Http.fetch, so allow the hosts they need first (allow_http_hosts).
        """

    def run(self, attributes: Union[str, Sequence[str], None] = None) -> str:
        """Run the functions with the given attribute(s) (#[main] when None)."""

    def lib(self, lib: str, name: str, func: Callable[..., Any], is_async: bool = False) -> None:
        """
        Add a Python function as a library function: `doc.lib('App', 'total', fn)` is `App.total(..)` (or
        `App::total(..)`) in Stof. It belongs to this document, and runs with the GIL on the thread making the call;
        it can't call back into the document that called it (that raises), but it can call other documents.
        """

    # Access (everything is off by default)

    def allow_system(self) -> None:
        """Give this document file system and environment access: the fs library, env functions, and file imports."""

    def allow_http(self) -> None:
        """Give this document network access (the Http library), to any public host."""

    def allow_private_network(self) -> None:
        """Let a document allowed any host (allow_http) reach this machine and private networks too."""

    def allow_http_hosts(self, hosts: List[str]) -> None:
        """
        Give this document network access to these hosts only: requests (and redirects) to any other host fail. An
        entry is a host name (any port, Ex. 'api.example.com') or a host and port ('localhost:8080').
        """

    def disallow_http_hosts(self, hosts: List[str]) -> bool:
        """
        Stop allowing these hosts (listed with allow_http_hosts). Removing every listed host leaves none allowed.
        Returns False when any host is allowed (allow_http without a list).
        """

    def allow_html_tags(self, tags: List[str]) -> None:
        """Elements the html format may write: only these, instead of the standard and custom elements."""

    def allow_html_attributes(self, attributes: List[str]) -> None:
        """Attributes the html format writes only when allowed: style, action, and formaction."""

    def keep_source(self, keep: bool = True) -> None:
        """Keep each function's source text from now on, so the readable `stof` export writes functions as Stof."""

    def quiet(self, quiet: bool = True) -> None:
        """Print nothing to the console: no parse warnings, test progress, or pln/err/dbg output."""

    def set_max_execution_time(self, seconds: Optional[float]) -> None:
        """How long a call can run, in seconds, before it fails (None for no limit). Defaults to 120."""

    def set_max_memory(self, bytes: Optional[int]) -> None:
        """How much memory a call can use, in bytes, before it fails (None for no limit). Defaults to 256 MB."""

    # Import and export

    def parse(self, stof: str, node: Optional[str] = None, profile: Profile = 'prod') -> bool:
        """
        Parse some Stof into this document (into node, an object path or ID, or the root). Profile 'untrusted' is for
        source from somewhere else: declarations only (nothing runs while it's parsed, no imports, no #[type]s);
        parse it into a fresh object, then seal that object confined before calling it.
        """

    def string_import(self, src: str, format: str = 'stof', node: Optional[str] = None, profile: Profile = 'prod') -> bool:
        """Import text in a format (Ex. 'stof', 'json', 'yaml', 'toml') into node, or the root."""

    def binary_import(self, bytes: bytes, format: str = 'bstf', node: Optional[str] = None, profile: Profile = 'prod') -> bool:
        """Import bytes in a format (Ex. 'bstf') into node, or the root."""

    def string_export(self, format: str = 'json', node: Optional[str] = None) -> str:
        """Export node (an object path or ID, the root by default) as text in a format (Ex. 'json', 'stof')."""

    def binary_export(self, format: str = 'bstf', node: Optional[str] = None) -> bytes:
        """Export node (an object path or ID, the root by default) as bytes in a format (Ex. 'bstf')."""

    # Sealing

    def seal(self, path: str, fields: bool = False, digest: Optional[str] = None, public_key: Optional[str] = None,
             signature: Optional[str] = None, removable: bool = False, reads: Optional[List[str]] = None,
             hosts: Optional[List[str]] = None, libs: Optional[List[str]] = None, time: Optional[float] = None,
             calls: Optional[List[str]] = None) -> str:
        """
        Seal an object (path, or a type as '<Name>') and everything under it: its functions can be called but not
        changed. With fields, its fields are frozen too. With digest, only if its digest is that; with public_key and
        signature, only if the digest is signed by that key (`Doc.sign(secret, 'stof/seal', digest)`). With
        removable, it can still be removed whole. With reads, its code reaches only itself and those objects; with
        hosts, its Http.fetch reaches only those hosts; with libs, it calls only those libraries; with time (ms), it
        runs for at most that long each time it's entered; with calls (function paths), those functions are its
        gates. Returns the digest.
        """

    def unseal(self, path: str) -> None:
        """Unseal an object that seal sealed (path, or a type as '<Name>')."""

    def digest(self, path: str, fields: bool = False) -> str:
        """The seal digest of an object (path, or a type as '<Name>'), with its fields' values when fields."""

    def sealed(self, path: str) -> Optional[Dict[str, Any]]:
        """How an object is sealed: a dict with part, fields, removable, and confined, or None when it isn't."""

    # Keys and text tools

    @staticmethod
    def generate_key() -> str:
        """A new secret key (base64, 32 bytes from the system's secure random source). Share only its public key."""

    @staticmethod
    def public_key(secret_key: str) -> str:
        """The public key (base64) for a secret key."""

    @staticmethod
    def sign(secret_key: str, context: str, message: Message) -> str:
        """
        Sign a message (str as UTF-8, or bytes) under a context (Ex. 'orders/v1') with a secret key: the signature
        (base64). It verifies only under the same context (in documents: Key.verify(public, context, message, sig)).
        """

    @staticmethod
    def verify(public_key: str, context: str, message: Message, signature: str) -> bool:
        """Is this a signature over this message (str or bytes), under this context, by this public key?"""

    @staticmethod
    def outline(text: str) -> List[Dict[str, Any]]:
        """
        The fields and functions in a Stof text: dicts with kind ('field' or 'fn'), path, name, object, line, col, and
        value (its source).
        """

    @staticmethod
    def edit(text: str, edits: List[Dict[str, Any]]) -> str:
        """
        Edit a Stof text in place: edits like {'set': 'a.b', 'value': 1}, {'set': 'a.b', 'stof': '1GB'},
        {'set_fn': 'a.f', 'stof': 'fn f() {}'}, {'remove': 'a.b'}, or {'rename': 'a.b', 'to': 'c'}.
        """
