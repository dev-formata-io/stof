# Markdown Library (Md)
Helper functions for common markdown operations, like turning Md strings into HTML strings.

# Md.html(md: str) -> str
Turn a markdown string into an HTML string.
```javascript
const md = '# Title\nList.\n- one\n- two';
const html = Md.html(md);
assert_eq(html, '<h1>Title</h1>\n<p>List.</p>\n<ul>\n<li>one</li>\n<li>two</li>\n</ul>');
```

# Md.json(md: str) -> str
Turn a markdown string into a JSON string.
```javascript
const md = '# Title\nList.\n- one\n- two';
const json = Md.json(md); // lots of info from the markdown parser
```

