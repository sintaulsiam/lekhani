//! Avro Keyboard 5 (.avrolayout) XML to Lekhani JSON Converter

use hashbrown::HashMap;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::{json, Value};
use std::path::Path;

pub fn convert_avro_layout_file<P: AsRef<Path>>(input_path: P) -> anyhow::Result<Value> {
    let content = std::fs::read_to_string(input_path.as_ref())?;
    convert_avro_layout_xml(&content)
}

pub fn convert_avro_layout_xml(xml_content: &str) -> anyhow::Result<Value> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut layout_name = String::from("Custom Layout");
    let mut layout_ver = String::from("1.0");
    let mut dev_name = String::from("Unknown");
    let mut dev_comment = String::new();
    let mut keys: HashMap<String, String> = HashMap::new();

    let mut buf = Vec::new();
    let mut current_element = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                current_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Event::Text(e) => {
                let txt = e.unescape()?.to_string();
                match current_element.as_str() {
                    "LayoutName" => layout_name = txt,
                    "LayoutVersion" => layout_ver = txt,
                    "DeveloperName" => dev_name = txt,
                    "DeveloperComment" => dev_comment = txt,
                    key if key.starts_with("Key_") || key.starts_with("Num") => {
                        let normalized_key = normalize_avro_key(key);
                        keys.insert(normalized_key, txt);
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                current_element.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    let json_val = json!({
        "info": {
            "type": "fixed",
            "version": "2",
            "layout": {
                "name": layout_name,
                "version": layout_ver,
                "developer": {
                    "name": dev_name,
                    "comment": dev_comment
                }
            }
        },
        "layout": keys
    });

    Ok(json_val)
}

fn normalize_avro_key(key: &str) -> String {
    let mut k = key.to_string();
    if k.contains("OEM1") {
        k = k.replace("OEM1", "Semicolon");
    }
    if k.contains("OEM2") {
        k = k.replace("OEM2", "Slash");
    }
    if k.contains("OEM3") {
        k = k.replace("OEM3", "BackQuote");
    }
    if k.contains("OEM4") {
        k = k.replace("OEM4", "OpenBracket");
    }
    if k.contains("OEM5") {
        k = k.replace("OEM5", "BackSlash");
    }
    if k.contains("OEM6") {
        k = k.replace("OEM6", "CloseBracket");
    }
    if k.contains("OEM7") {
        k = k.replace("OEM7", "Quote");
    }
    if k.contains("MINUS") {
        k = k.replace("MINUS", "Minus");
    }
    if k.contains("PLUS") {
        k = k.replace("PLUS", "Equals");
    }
    if k.contains("PERIOD") {
        k = k.replace("PERIOD", "Period");
    }
    if k.contains("COMMA") {
        k = k.replace("COMMA", "Comma");
    }
    k
}
