
import { StofDoc } from '../doc.ts';
const doc = await StofDoc.new();

// Stof Std pln function mapped to console.log
doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));

// My example nested function that is async, mapped to an async Stof lib fn
doc.lib('Example', 'nested', async (): Promise<Map<string, string>> => {
    const res = new Map();
    res.set('msg', 'hello, there');
    res.set('nested', await (async (): Promise<string> => 'this is a nested async JS fn (like fetch)')());
    return res;
}, true);

// Add some Stof
doc.parse(`
    #[my_main]
    fn main() {
        const res = await Example.nested();
        pln(res);
    }
`);
await doc.run('my_main'); // default is 'main'
