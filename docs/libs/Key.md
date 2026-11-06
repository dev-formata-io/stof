# Key Library (Key)
Signatures: prove who wrote something, and that it hasn't changed. Make a secret key (`Key::generate`), share its public key (`Key::public`), sign the exact bytes of a message (`Key::sign`), and check a signature (`Key::verify`). Ed25519, with keys and signatures as base64 strings, so they travel in JSON, environment variables, and headers. Hosts have the same functions (`StofDoc.sign`, `Doc.sign`, `stof::model::seal::sign`), so a signature made in one place checks in any other.

Every signature is made under a context, a short name for what it's for (Ex. "orders/v1"), and only verifies under that same context. A seal signature is a signature over a part's digest (`digest(obj)`) under the context "stof/seal", which hosts check with `doc.seal(path, { publicKey, signature })`.

Keys for signing are separate from Age keys (which encrypt): use both to send something only one reader can open and that they know came from you.

## Example Usage
```rust
#[main]
fn main() {
    const secret = Key::generate();       // keep this one secret
    const public = Key::public(secret);   // share this one

    const order = '{"item":"lamp","qty":2}';
    const signature = Key::sign(secret, 'orders/v1', order);

    pln(Key::verify(public, 'orders/v1', order, signature));                    // true
    pln(Key::verify(public, 'orders/v1', order.replace('2', '20'), signature)); // false: changed
    pln(Key::verify(public, 'refunds/v1', order, signature));                   // false: another purpose
}
```

# Key::generate() -> str
A new secret key: 32 bytes from the system's secure random source, as base64. Keep it secret (pass it in where it's needed, or encrypt it with Age if a document has to hold it), and share only its public key (`Key::public`).
```rust
const secret = Key::generate();
const public = Key::public(secret);
```


# Key::public(secret: str) -> str
The public key for a secret key (base64). Anyone with it can check your signatures (`Key::verify`), but can't make them.
```rust
const public = Key::public(secret);
```


# Key::sign(secret: str, context: str, message: str | blob) -> str
Sign a message with a secret key: the signature (base64). What's signed is the message's exact bytes (a string's UTF-8, or a blob), so sign the text you send (Ex. `stringify('json', order)`), not an object.

The context names what the signature is for (Ex. "orders/v1"): it only verifies under that same context, so a signature made for one purpose can't be passed off as another. A seal signature is a signature over a part's digest under the context "stof/seal".
```rust
const order = stringify('json', self.order);
const signature = Key::sign(secret, 'orders/v1', order);
assert(Key::verify(Key::public(secret), 'orders/v1', order, signature));

// sign a part of the document, for a host to check when it seals it
const part = Key::sign(secret, 'stof/seal', digest(self.rules));
```


# Key::verify(public: str, context: str, message: str | blob, signature: str) -> bool
Is `signature` a signature over exactly these bytes, under this context, by the secret key behind `public`? False for anything else: another message (even one byte), another context, another key, or a key or signature that isn't one.
```rust
fn accept(order: str, signature: str) -> bool {
    if (!Key::verify(self.shop_key, 'orders/v1', order, signature)) return false;
    parse(order, self.order, 'json'); // trusted now
    true
}
```


