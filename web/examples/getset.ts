import { StofDoc } from '../doc.ts';
const doc = await StofDoc.new();

doc.parse(`
    field: 42
    
    #[type]
    StaticVars: {
        another: 30
    }
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