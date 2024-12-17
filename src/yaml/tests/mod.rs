//
// Copyright 2024 Formata, Inc. All rights reserved.
//

use crate::SDoc;


#[test]
fn import_export() {
    let mut doc = SDoc::file("src/yaml/tests/example.yaml", "yaml").unwrap();
    assert_eq!(doc.export_string("json", None).unwrap(), "{\"Hello\":[\"dude\",\"hi\",\"c\"]}");
    assert_eq!(doc.export_string("yaml", None).unwrap(), "Hello:\n- dude\n- hi\n- c\n");
    assert_eq!(doc.export_string("toml", None).unwrap(), "Hello = [\"dude\", \"hi\", \"c\"]\n");
    assert_eq!(doc.export_string("urlencoded", None).unwrap(), "Hello%5B0%5D=dude&Hello%5B1%5D=hi&Hello%5B2%5D=c");
    assert_eq!(doc.export_string("xml", None).unwrap(), "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Hello>dude</Hello><Hello>hi</Hello><Hello>c</Hello>");

    let _ = doc.string_import("urlencoded", "value=true&another=false", "Import");
    assert_eq!(doc.export_string("json", doc.graph.node_ref("Import", None).as_ref()).unwrap(), "{\"another\":false,\"value\":true}");
}
