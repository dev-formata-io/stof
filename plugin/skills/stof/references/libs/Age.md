# Age

# Age.blobify(recipients: str | list | Data<Age>, format: str = 'stof', context?: obj) -> blob
Std.blobify, but with age public-key recipients. The resulting blob can only be parsed by a recipient's private key.


# Age.generate(context: obj = self) -> Data<Age>
Generate a new Age Identity (Data<Age>) on the given context object (default is self).


# Age.parse(age: Data<Age>, bin: blob, context: obj = self, format: str = "stof") -> bool
Parse an age-encrypted binary. Similar to Std.parse, but requires an Age identity (secret private key).


# Age.pass_blobify(passphrase: str, format: str = 'stof', context?: obj) -> blob
Std.blobify, but with an age passphrase recipient. The resulting blob can only be parsed with the provided passphrase.


# Age.pass_parse(passphrase: str, bin: blob, context: obj = self, format: str = "stof") -> bool
Parse an age-encrypted binary with a passphrase. Similar to Std.parse, but requires a passphrase for decryption.


# Age.public(age: Data<Age>) -> str
Get the public key for a given age identity.


