import { StofDoc } from '../doc.ts';
const doc = await StofDoc.new();

doc.lib('fs', 'read_string', (path: string): string => {
    console.log(path);
    const res = Deno.readTextFileSync(path);
    console.log(res);
    return res;
});

doc.parse(`
    field: 42
    
    #[type]
    StaticVars: {
        another: 30
    }

    import 'C:/Users/cummi/Dev/Formata/github/stof/pkg.stof' as self.Imported;
`);

const field = doc.get('field');
console.log(field); // 42

const success = doc.set('field', 77);
console.log(success); // true
console.log(doc.get('field')); // 77

const another = doc.get('<StaticVars>.another');
console.log(another); // 30

const anotherSuccess = doc.set('<StaticVars>.another', 56);
console.log(anotherSuccess); // true
console.log(doc.get('<StaticVars>.another')); // 56

const imported = doc.get('root.Imported.import');
console.log(imported);