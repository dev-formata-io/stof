# Age

# Age::blobify(recipients: str | list | Data<Age>, format: str = 'stof', context?: obj) -> blob
Export `context` (like `Std::blobify`) encrypted to one or more recipients: their public keys (`age1...`, from `Age::public`), or identities (`Data<Age>`). Only a recipient's secret key can open it (`Age::parse`). To also prove who sent it, sign the blob (`Key::sign`).


# Age::generate(context: obj = self) -> Data<Age>
A new Age identity (an encryption key pair), kept on `context` as `Data<Age>`: its secret never prints, but it is part of the object (a BSTF export of the object carries it). Share `Age::public(identity)`. Encryption keys are separate from signing keys (`Key`).


# Age::parse(age: Data<Age> | str, bin: blob, context: obj = self, format: str = "stof") -> bool
Decrypt an age-encrypted blob and parse it into `context` (like `Std::parse`), with the identity it was encrypted to: a `Data<Age>` from `Age::generate`, or the secret key as a string (`AGE-SECRET-KEY-1...`, Ex. passed in by the host). False when the blob wasn't encrypted to this identity.
```rust
const me = Age::generate();
const blob = Age::blobify(Age::public(me), 'json', self.message);
assert(Age::parse(me, blob, self.received, 'json'));
```


# Age::pass_blobify(passphrase: str, format: str = 'stof', context?: obj) -> blob
Export `context` (like `Std::blobify`) encrypted with a passphrase. Only the same passphrase opens it (`Age::pass_parse`). A way to keep secrets (Ex. a signing key) in a document at rest.


# Age::pass_parse(passphrase: str, bin: blob, context: obj = self, format: str = "stof") -> bool
Parse an age-encrypted binary with a passphrase. Similar to Std.parse, but requires a passphrase for decryption.


# Age::public(age: Data<Age> | str) -> str
The public key (`age1...`) for an identity (`Data<Age>`, or its secret key as a string). Anyone can encrypt to it (`Age::blobify`); only the identity can decrypt.


