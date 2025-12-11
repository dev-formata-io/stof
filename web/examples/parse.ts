import { StofDoc } from "../doc.ts";

const doc = await StofDoc.parse({
    name: 'CJ',
    field: 42
});

doc.parse(`
    instant: {
        days ttl: Time.now()
    }

    fn say_hi() -> str {
        self.toml_field ?? 'hi, there'
    }
`);

doc.parse(`
toml_field = "hello"
`, 'toml');

console.log('SAY HI:', await doc.call('say_hi'));
console.log('\nTOML:\n', doc.stringify('toml'));
console.log('\nRECORD:\n', doc.record());
