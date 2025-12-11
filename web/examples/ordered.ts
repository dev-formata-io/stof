import { stof, StofDoc } from "../doc.ts";
await StofDoc.initialize();

const doc = stof`
#[type]
#[no-export]
Checkbox: {
    checked: false
    text: ''

    fn click() {
        self.checked = !self.checked;
    }
}

language: 'en'
document: {
    title: 'To-Do'
    main: {
        list: [
            {
                text: 'Try ordering'
            } as Checkbox,
            {
                text: 'Do a thing'
            } as Checkbox
        ]
    }
}

#[click]
fn click_it() {
    ?self.document.main.list.back().click();
}
`;

await doc.run('click');
console.log(doc.stringify('yaml'));

/* deno run --allow-all web/examples/ordered.ts
language: en
document:
  title: To-Do
  main:
    list:
    - text: Try ordering
      checked: false
    - text: Do a thing
      checked: true
*/
