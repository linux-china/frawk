use std::collections::HashMap;
use miniserde::json;
use miniserde::json::{Number, Value};
use crate::runtime::{Str, StrMap};

pub fn to_json(obj: &StrMap<Str>) -> String {
    let mut json_obj: HashMap<String, Value> = HashMap::new();
    obj.iter(|map| {
        for (key, value) in map {
            if !value.is_empty() {
                let value_text = value.to_string();
                if value_text.contains('.') { // check float
                    if let Ok(num) = value_text.parse::<f64>() {
                        json_obj.insert(key.to_string(), Value::Number(Number::F64(num)));
                    } else {
                        json_obj.insert(key.to_string(), Value::String(value_text));
                    }
                } else { // check integer
                    if let Ok(num) = value_text.parse::<i64>() {
                        json_obj.insert(key.to_string(), Value::Number(Number::I64(num)));
                    } else {
                        json_obj.insert(key.to_string(), Value::String(value_text));
                    }
                }
            }
        }
    });
    json::to_string(&json_obj)
}

pub fn from_json(json_text: &str) -> StrMap<Str> {
    let mut map = hashbrown::HashMap::new();
    if let Ok(json_obj) = json::from_str::<HashMap<String,Value>>(json_text) {
        for (key, value) in json_obj {
           match value {
                Value::Bool(b) => {
                    if b {
                        map.insert(Str::from(key), Str::from("1"));
                    } else {
                        map.insert(Str::from(key), Str::from("0"));
                    }
                }
                Value::Number(num) => {
                    map.insert(Str::from(key), Str::from(num.to_string()));
                }
                Value::String(s) => {
                    map.insert(Str::from(key), Str::from(s));
                }
                _ => {}
            }
        }
    }
    return StrMap::from(map);
}