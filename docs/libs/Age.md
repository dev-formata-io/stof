# Age

# Age.blobify(recipients: str | list | Data<Age>, format: str = 'stof', context?: obj) -> blob
Std.blobify, but with age public-key recipients. The resulting blob can only be parsed by a recipient's private key.


# Age.generate(context: obj = self) -> Data<Age>
Generate a new Age Identity (Data<Age>) on the given context object (default is self).


# Age.parse(age: Data<Age>, bin: blob, context: obj = self, format: str = "stof") -> bool
Parse an age-encrypted binary. Similar to Std.parse, but requires an Age identity (secret private key).


# Age.public(age: Data<Age>) -> str
Get the public key for a given age identity.


