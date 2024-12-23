//
// Copyright 2024 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use core::str;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde_json::{Map, Value};
use crate::{json::JSON, SGraph};


/// Read XML string to SGraph.
/// Warning - looses original XML formatting. 
pub fn read_xml_to_graph(xml: &str) -> SGraph {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut value_stack = Vec::new();
    value_stack.push(Value::Object(Map::new())); // Root object!

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            // Handle errors and end of file
            Err(e) => {
                panic!("Error at position {}: {:?}", reader.buffer_position(), e);
            },
            Ok(Event::Eof) => break,

            // Empty xml tag
            Ok(Event::Empty(e)) => {
                let name = e.name();
                let tag = std::str::from_utf8(name.as_ref()).unwrap().to_string();

                // Get all attributes from this tag
                let mut attributes_map = Map::new();
                for attr in e.attributes() {
                    match attr {
                        Ok(attr) => {
                            let name = std::str::from_utf8(attr.key.0).unwrap();
                            let value = std::str::from_utf8(attr.value.as_ref()).unwrap();
                            attributes_map.insert(name.to_owned(), Value::String(value.to_owned()));
                        },
                        _ => {}
                    }
                }

                // Create the value to add - always an object for empty tags!?
                let mut value = Value::Object(Map::new());
                if attributes_map.len() > 0 {
                    if let Some(map) = value.as_object_mut() {
                        map.insert("attributes".to_string(), Value::Object(attributes_map));
                    }
                }

                // Insert the value into the previous object! (if able)
                if let Some(prev) = value_stack.last_mut() {
                    if let Some(map) = prev.as_object_mut() {
                        if let Some(existing) = map.get_mut(&tag) {
                            if existing.is_array() {
                                let arr = existing.as_array_mut().unwrap();
                                arr.push(value);
                            } else {
                                value = Value::Array(vec![value, existing.clone()]);
                                map.insert(tag, value);
                            }
                        } else {
                            map.insert(tag, value);
                        }
                    }
                } else {
                    // This is the top level object!
                    value_stack.push(value);
                }
            },

            // Start of an XML tag
            Ok(Event::Start(e)) => {
                // Assume this tag is an object to start with
                value_stack.push(Value::Object(Map::new()));

                // Get all attributes from this tag
                let mut attributes_map = Map::new();
                for attr in e.attributes() {
                    match attr {
                        Ok(attr) => {
                            let name = std::str::from_utf8(attr.key.0).unwrap();
                            let value = std::str::from_utf8(attr.value.as_ref()).unwrap();
                            attributes_map.insert(name.to_owned(), Value::String(value.to_owned()));
                        },
                        _ => {}
                    }
                }

                // Insert attributes object if needed
                if attributes_map.len() > 0 {
                    if let Some(last) = value_stack.last_mut() {
                        if let Some(map) = last.as_object_mut() {
                            map.insert("attributes".to_string(), Value::Object(attributes_map));
                        }
                    }
                }
            }

            // Text part of a tag
            Ok(Event::Text(e)) => {
                let text_name = "text".to_string();
                let mut text = e.unescape().unwrap().into_owned();
                if let Some(last) = value_stack.last_mut() {
                    if let Some(map) = last.as_object_mut() {
                        if map.contains_key(&text_name) {
                            if let Some(txt) = map.get_mut(&text_name) {
                                let mut current = txt.as_str().unwrap().to_owned();
                                current.push_str(&text);
                                text = current;
                            }
                        }
                        map.insert(text_name, Value::String(text));
                    }
                }
            },

            // Comment part of a tag
            Ok(Event::Comment(e)) => {
                let text_name = "comments".to_string();
                let text = e.unescape().unwrap().into_owned();
                if let Some(last) = value_stack.last_mut() {
                    if let Some(map) = last.as_object_mut() {
                        if map.contains_key(&text_name) {
                            if let Some(array_value) = map.get_mut(&text_name) {
                                let array = array_value.as_array_mut().unwrap();
                                array.push(Value::String(text));
                            }
                        } else {
                            map.insert(text_name, Value::Array(vec![Value::String(text)]));
                        }
                    }
                }
            }

            // Processing instruction
            Ok(Event::PI(e)) => {
                let text_name = "processing".to_string();
                let text = str::from_utf8(&e).expect("PI is not utf-8").to_string();
                if let Some(last) = value_stack.last_mut() {
                    if let Some(map) = last.as_object_mut() {
                        if map.contains_key(&text_name) {
                            if let Some(array_value) = map.get_mut(&text_name) {
                                let array = array_value.as_array_mut().unwrap();
                                array.push(Value::String(text));
                            }
                        } else {
                            map.insert(text_name, Value::Array(vec![Value::String(text)]));
                        }
                    }
                }
            },

            // End of an xml tag
            Ok(Event::End(e)) => {
                let name = e.name();
                let tag = std::str::from_utf8(name.as_ref()).unwrap().to_string();

                let last_val_obj = value_stack.pop().unwrap();
                let last_val = last_val_obj.as_object().unwrap();
                let mut value: Value;
                if last_val.len() < 2 && value_stack.len() > 0 {
                    // This is a VALUE?, not an object, so unpack it! (if able)
                    if let Some(text) = last_val.get(&"text".to_string()) {
                        value = text.clone();
                    } else {
                        value = last_val_obj;
                    }
                } else {
                    // This is an object
                    value = last_val_obj;
                }

                // Insert the value into the previous object! (if able)
                if let Some(prev) = value_stack.last_mut() {
                    if let Some(map) = prev.as_object_mut() {
                        if let Some(existing) = map.get_mut(&tag) {
                            if existing.is_array() {
                                let arr = existing.as_array_mut().unwrap();
                                arr.push(value);
                            } else {
                                value = Value::Array(vec![value, existing.clone()]);
                                map.insert(tag, value);
                            }
                        } else {
                            map.insert(tag, value);
                        }
                    }
                } else {
                    // This is the top level object!
                    value_stack.push(value);
                }
            },
            _ => (),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    if value_stack.len() != 1 {
        panic!("Not valid XML - found too many or not enough ending tags");
    }
    let value = value_stack.pop().unwrap();
    JSON::from_value(value)
}


#[cfg(test)]
mod tests {
    use crate::SDoc;

    #[test]
    fn test() {
        let xml = r#"
<point type="2d">
    <x>
        <!--X Position of the point-->
        <!--Should be less than 1000-->
        12.2345
    </x>
    <y>
        <!--Y Position of the point-->
        18.63433
    </y>
    <xx>12.2345</xx>
    <yy>18.63433</yy>
    <z type="3D_na"/>
    <?xml-stylesheet href = "tutorialspointstyle.css" type = "text/css"?>
    <?xml-stylesheet href = "tutorialspointstyle.css" type = "text/css"?>
</point>"#;

        let doc = SDoc::src(xml, "xml").unwrap();
        assert_eq!(doc.export_string("main", "json", None).unwrap(), "{\"point\":{\"attributes\":{\"type\":\"2d\"},\"processing\":[\"xml-stylesheet href = \\\"tutorialspointstyle.css\\\" type = \\\"text/css\\\"\",\"xml-stylesheet href = \\\"tutorialspointstyle.css\\\" type = \\\"text/css\\\"\"],\"x\":{\"comments\":[\"X Position of the point\",\"Should be less than 1000\"],\"text\":\"12.2345\"},\"xx\":\"12.2345\",\"y\":{\"comments\":[\"Y Position of the point\"],\"text\":\"18.63433\"},\"yy\":\"18.63433\",\"z\":{\"attributes\":{\"type\":\"3D_na\"}}}}");
    }
}
