# PDF Library (Pdf)
Functions for working with PDF files, loaded into Stof via the custom Data<Pdf> type. Requires the "pdf" feature flag to be enabled.

# Pdf.extract_images(pdf: Data\<Pdf>) -> list
Given a data pointer to a PDF document, extract all images from every page, returning them as a list of maps with image data.
```rust
// import './test_stof_pdf.pdf'; // taken from stof PDF format tests
const images = self.pdf.extract_images();
assert_eq(images.len(), 1);
assert_eq(images[0].get('height'), 500);
assert_eq(images[0].get('width'), 1250);
```


# Pdf.extract_text(pdf: Data\<Pdf>) -> str
Given a data pointer to a PDF document, extract all text from the PDF file and return it as a string.
```rust
// import './test_stof_pdf.pdf'; // taken from stof PDF format tests
const text = self.pdf.extract_text();
assert_eq(text, "Example Stof\nDocument\n");
```


