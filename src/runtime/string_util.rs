use pad::{Alignment, PadStr};
use crate::runtime::{SharedMap, Str, StrMap};

pub fn pad_left(text: &str, len: usize, pad: &str) -> String {
    if text.len() > len {
        return text[0..len].to_string();
    }
    let pad_char = pad.chars().next().unwrap();
    text.pad(len, pad_char, Alignment::Left, false)
}

pub fn pad_right(text: &str, len: usize, pad: &str) -> String {
    if text.len() > len {
        return text[0..len].to_string();
    }
    let pad_char = pad.chars().next().unwrap();
    text.pad(len, pad_char, Alignment::Right, false)
}

pub fn pad_both(text: &str, len: usize, pad: &str) -> String {
    if text.len() > len {
        return text[0..len].to_string();
    }
    let pad_char = pad.chars().next().unwrap();
    text.pad(len, pad_char, Alignment::MiddleRight, false)
}

pub fn strcmp(text1: &str, text2: &str) -> i64 {
    return if text1 == text2 {
        0
    } else if text1 < text2 {
        -1
    } else {
        1
    };
}

pub fn read_all(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

pub fn write_all(path: &str, content: &str) {
    std::fs::write(path, content).unwrap()
}

pub(crate) fn pairs<'a>(text: &str, pair_sep: &str, kv_sep: &str) -> StrMap<'a, Str<'a>> {
    let is_url_query = pair_sep == "&" && kv_sep == "=";
    let mut map = hashbrown::HashMap::new();
    text.trim_matches(|c| c == '"' || c == '\'').split(pair_sep).for_each(|pair| {
        let kv: Vec<&str> = pair.split(kv_sep).collect();
        if kv.len() == 2 && !kv[1].is_empty() {
            let mut value = kv[1].to_string();
            if is_url_query {
                if let Ok(param_value) = urlencoding::decode(kv[1]) {
                    value = param_value.to_string();
                }
            }
            map.insert(Str::from(kv[0].to_string()), Str::from(value));
        }
    });
    SharedMap::from(map)
}

use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
enum AttributesToken<'a> {
    #[token("{")]
    LBRACE,
    #[token("}")]
    RBRACE,
    #[token(",")]
    COMMA,
    #[token("=")]
    EQ,
    #[token(":")]
    COLON,
    #[token(";")]
    SEMICOLON,
    #[regex(r#"[a-zA-Z0-9_]*"#)]
    LITERAL(&'a str),
    #[regex(r#""[^"]*""#)]
    Text(&'a str),
    #[regex(r#"'[^']*'"#)]
    Text2(&'a str),
    #[regex(r#"(\d+)(\.\d+)?"#)]
    NUM(&'a str),
}

struct PairState {
    pub key: String,
    pub value: String,
    pub key_parsed: bool,
}

impl Default for PairState {
    fn default() -> Self {
        PairState {
            key: "".to_owned(),
            value: "".to_owned(),
            key_parsed: false,
        }
    }
}

impl PairState {
    pub fn reset(&mut self) {
        self.key = "".to_owned();
        self.value = "".to_owned();
        self.key_parsed = false;
    }

    pub fn is_legal(&self) -> bool {
        self.key != "" && self.value != ""
    }
}

pub(crate) fn attributes(text: &str) -> StrMap<Str> {
    let mut map = hashbrown::HashMap::new();
    if text.contains('{') {
        let offset = text.find('{').unwrap();
        let name = text[0..offset].trim().to_string();
        map.insert(Str::from("_".to_owned()), Str::from(name));
        let pairs_text = text[offset..].to_string();
        let mut pair_state = PairState::default();
        let lexer = AttributesToken::lexer(&pairs_text);
        for token in lexer.into_iter() {
            if let Ok(attribute) = token {
                match attribute {
                    AttributesToken::COLON | AttributesToken::EQ => {
                        pair_state.key_parsed = true;
                    }
                    AttributesToken::LITERAL(literal) => {
                        if pair_state.key_parsed {
                            pair_state.value = literal.to_string();
                            if pair_state.is_legal() {
                                map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                            }
                            pair_state.reset();
                        } else {
                            pair_state.key = literal.to_string();
                        }
                    }
                    AttributesToken::Text(text) => {
                        pair_state.value = text[1..text.len() - 1].to_string();
                        if pair_state.is_legal() {
                            map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                        }
                        pair_state.reset();
                    }
                    AttributesToken::Text2(text) => {
                        pair_state.value = text[1..text.len() - 1].to_string();
                        if pair_state.is_legal() {
                            map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                        }
                        pair_state.reset();
                    }
                    AttributesToken::NUM(num) => {
                        pair_state.value = num.to_string();
                        if pair_state.is_legal() {
                            map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                        }
                        pair_state.reset();
                    }
                    _ => {}
                }
            }
        }
    } else {
        map.insert(Str::from("_".to_owned()), Str::from(text));
    }
    SharedMap::from(map)
}


#[cfg(test)]
mod tests {
    use unicode_segmentation::UnicodeSegmentation;
    use super::*;

    #[test]
    fn test_pad_left() {
        let text = pad_left("hello", 100, "*");
        println!("{}", text);
    }

    #[test]
    fn test_strcmp() {
        let text1 = "hello";
        let text2 = "Hello";
        println!("{}", strcmp(text1, text2));
    }

    #[test]
    fn test_words() {
        let text = "Hello , world! could you give a 名称?";
        let words = text.unicode_words();
        for word in words {
            println!("{}", word);
        }
    }

    #[test]
    fn test_repeat() {
        let text = "12";
        let result = text.repeat(3);
        println!("{}", result);
    }

    #[test]
    fn test_read_all() {
        let content = read_all("demo.awk");
        println!("{}", content);
    }

    #[test]
    fn test_write_all() {
        let content = "hello";
        write_all("demo2.txt", content);
        write_all("demo2.txt", "hello2");
    }

    #[test]
    fn test_pairs() {
        let text = "name=hello;age=12";
        let map = pairs(text, ";", "=");
        println!("{:?}", map);
    }

    #[test]
    fn test_attributes() {
        let text = r#"mysql{host=localhost user=root password=123456 database=test}"#;
        let map = attributes(text);
        println!("{}", map.get(&Str::from("_")).as_str());
        println!("{}", map.get(&Str::from("host")).as_str());
        println!("{}", map.get(&Str::from("user")).as_str());
    }

    #[test]
    fn test_complex_attributes() {
        let text = r#"http_requests_total{method="hello 你好 ' = : , world",code=200.01}"#;
        let map = attributes(text);
        println!("{}", map.get(&Str::from("method")).as_str());
        println!("{}", map.get(&Str::from("code")).as_str());
    }
}
