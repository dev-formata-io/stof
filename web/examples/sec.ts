import { StofDoc } from '../doc.ts';
const doc = await StofDoc.new();
doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));

// Example is in src/model/formats/stof/tests/security/age.stof
// This example verifies that Age is working in WASM
doc.parse(`
Receiver: {
    #[private]
    passport: Age.generate()

    fn key() -> str {
        self.passport.public()
    }

    fn receive(encrypted: blob) -> str {
        const dest = new {};
        Age.parse(self.passport, encrypted, dest, 'bstf');
        ?dest.speak();
        dest.message ?? 'none'
    }
}


#[main]
fn sending_sensitive() {
    // Create a binary blob of encrypted data that only the receiver can use
    const encrypted: blob = {
        // Create the payload, including Stof functions/APIs
        const payload = new { message: 'hey there, this is secret' };
        parse(r#"fn speak() { pln('Speaking: ', self.message); }"#, payload, 'stof');

        // Can use one or more receivers Data<Age> or str public keys (list for multiple)
        const res = Age.blobify(self.Receiver.key(), 'bstf', payload);
        drop(payload);
        res
    };

    // Now have the receiver do what it wants with it
    const msg = self.Receiver.receive(encrypted);
    assert_eq(msg, 'hey there, this is secret');
}
`);
await doc.run();
