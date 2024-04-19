use pad::{Alignment, PadStr};
use crate::runtime::{IntMap, SharedMap, Str, StrMap};

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
    let mut reader = oneio::get_reader(path).unwrap();
    let mut text = "".to_string();
    reader.read_to_string(&mut text).unwrap();
    text
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
enum RecordToken<'a> {
    #[token("{")]
    LBRACE,
    #[token("}")]
    RBRACE,
    #[token("(")]
    LPAREN,
    #[token(")")]
    RPAREN,
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

/// parse message - `msg_name{key1=value1,key2=value2}(body)`
pub(crate) fn message(text: &str) -> StrMap<Str> {
    record(text)
}

/// parse record: `attr_name{key1=value1,key2=value2}`
pub(crate) fn record(text: &str) -> StrMap<Str> {
    let mut map = hashbrown::HashMap::new();
    // table definition: table_name(id integer, value double)
    if text.contains("(") && text.ends_with(")") {
        let offset = text.find('(').unwrap();
        let name = text[0..offset].trim().to_string();
        if !name.is_empty() {
            map.insert(Str::from("_".to_owned()), Str::from(name));
        }
        let pairs_text = text[offset + 1..text.len() - 1].to_string();
        for pair in pairs_text.split(",") {
            let kv: Vec<&str> = pair.trim().split(" ").collect();
            if kv.len() == 2 {
                map.insert(Str::from(kv[0].trim().to_string()), Str::from(kv[1].trim().to_string()));
            }
        }
    } else if text.contains('{') { // record_name{id=1,name="hello world"}
        let offset = text.find('{').unwrap();
        let name = text[0..offset].trim().to_string();
        if !name.is_empty() {
            map.insert(Str::from("_".to_owned()), Str::from(name));
        }
        let pairs_text = text[offset..].to_string();
        let mut pair_state = PairState::default();
        let mut body_started = false;
        let mut body = "".to_owned();
        let lexer = RecordToken::lexer(&pairs_text);
        for token in lexer.into_iter() {
            if let Ok(attribute) = token {
                match attribute {
                    RecordToken::COLON | RecordToken::EQ => { // key parsed
                        pair_state.key_parsed = true;
                    }
                    RecordToken::LPAREN => { // body started
                        body_started = true;
                    }
                    RecordToken::RPAREN => { // boyd end
                        if !body.is_empty() {
                            map.insert(Str::from("_body".to_owned()), Str::from(body.clone()));
                        }
                        body_started = false;
                    }
                    // parse key's value
                    RecordToken::LITERAL(literal) if !body_started => { // pair value
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
                    RecordToken::Text(text) if !body_started => { // pair value
                        pair_state.value = text[1..text.len() - 1].to_string();
                        if pair_state.is_legal() {
                            map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                        }
                        pair_state.reset();
                    }
                    RecordToken::Text2(text) if !body_started => { // pair value
                        pair_state.value = text[1..text.len() - 1].to_string();
                        if pair_state.is_legal() {
                            map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                        }
                        pair_state.reset();
                    }
                    RecordToken::NUM(num)  if !body_started => { // pair value
                        pair_state.value = num.to_string();
                        if pair_state.is_legal() {
                            map.insert(Str::from(pair_state.key.clone()), Str::from(pair_state.value.clone()));
                        }
                        pair_state.reset();
                    }
                    // body value
                    RecordToken::LITERAL(literal) if body_started => {
                        body = literal.to_string();
                    }
                    RecordToken::NUM(num)  if body_started => {
                        body = num.to_string();
                    }
                    RecordToken::Text(text) if body_started => {
                        body = text[1..text.len() - 1].to_string();
                    }
                    RecordToken::Text2(text) if body_started => {
                        body = text[1..text.len() - 1].to_string();
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

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
enum ParamsToken<'a> {
    #[token("(")]
    LBRACE,
    #[token(")")]
    RBRACE,
    #[token(",")]
    COMMA,
    #[regex(r#"[a-zA-Z0-9_]*"#)]
    LITERAL(&'a str),
    #[regex(r#""[^"]*""#)]
    Text(&'a str),
    #[regex(r#"'[^']*'"#)]
    Text2(&'a str),
    #[regex(r#"(\d+)(\.\d+)?"#)]
    NUM(&'a str),
}

pub(crate) fn func<'a>(text: &str) -> IntMap<Str<'a>> {
    let result: IntMap<Str> = IntMap::default();
    if text.contains("(") {
        let offset = text.find('(').unwrap();
        let name = text[0..offset].trim().to_string();
        result.insert(0, Str::from(name));
        let params_text = text[offset..].to_string();
        let lexer = ParamsToken::lexer(&params_text);
        let mut index: i64 = 1;
        for token in lexer.into_iter() {
            if let Ok(attribute) = token {
                match attribute {
                    ParamsToken::LITERAL(literal) => {
                        result.insert(index, Str::from(literal.to_string()));
                        index = index + 1;
                    }
                    ParamsToken::Text(text) => {
                        result.insert(index, Str::from(text[1..text.len() - 1].to_string()));
                        index = index + 1;
                    }
                    ParamsToken::Text2(text) => {
                        result.insert(index, Str::from(text[1..text.len() - 1].to_string()));
                        index = index + 1;
                    }
                    ParamsToken::NUM(num) => {
                        result.insert(index, Str::from(num.to_string()));
                        index = index + 1;
                    }
                    _ => {}
                }
            }
        }
    }
    result
}

pub fn last_part(text: &str, sep: &str) -> String {
    if !sep.is_empty() {
        let parts: Vec<&str> = text.split(sep).collect();
        if parts.len() > 0 {
            return parts[parts.len() - 1].to_string();
        }
    } else {
        if text.contains('/') {
            let parts: Vec<&str> = text.split('/').collect();
            if parts.len() > 0 {
                return parts[parts.len() - 1].to_string();
            }
        } else if text.contains('.') {
            let parts: Vec<&str> = text.split('.').collect();
            if parts.len() > 0 {
                return parts[parts.len() - 1].to_string();
            }
        }
    }
    text.to_string()
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
    fn test_read_all_from_remote() {
        let content = read_all("https://httpbin.org/ip");
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
    fn test_record() {
        let text = r#"mysql{host=localhost user=root password=123456 database=test}(1)"#;
        let map = record(text);
        println!("{}", map.get(&Str::from("_")).as_str());
        println!("{}", map.get(&Str::from("host")).as_str());
        println!("{}", map.get(&Str::from("user")).as_str());
        println!("body: {}", map.get(&Str::from("_body")).as_str());
    }

    #[test]
    fn test_table_record() {
        let text = r#"table1(id int, age int)"#;
        let map = record(text);
        println!("{}", map.get(&Str::from("_")).as_str());
        println!("{}", map.get(&Str::from("id")).as_str());
    }

    #[test]
    fn test_cookies() {
        let cookies_text = "_octo=GH1.1.178216615.1688558702; preferred_color_mode=light; tz=Asia%2FShanghai; _device_id=c49fdb13b5c41be361ee80236919ba50; user_session=qDSJ7GlA3aLriNnDG-KJsqw_QIFpmTBjt0vcLy5Vq2ay6StZ; __Host-user_session_same_site=qDSJ7GlA3aLriNnDG-KJsqw_QIFpmTBjt0vcLy5Vq2ay6StZ; tz=Asia%2FShanghai;";
        let cookies = pairs(cookies_text, ";", "=");
        println!("{}", cookies.get(&Str::from("_octo")).as_str());
        println!("{}", cookies.get(&Str::from("preferred_color_mode")).as_str());
        println!("{}", cookies.get(&Str::from("tz")).as_str());
    }

    #[test]
    fn test_complex_record() {
        let text = r#"http_requests_total{method="hello 你好 ' = : , world",code=200.01}"#;
        let map = record(text);
        println!("{}", map.get(&Str::from("method")).as_str());
        println!("{}", map.get(&Str::from("code")).as_str());
    }

    #[test]
    fn test_func() {
        let func_text = "hello(x,'hello world',11)";
        let map = func(func_text);
        let len = map.len();
        for i in 0..len {
            println!("{}", map.get(&(i as i64)).as_str());
        }
    }

    #[test]
    fn test_last_part() {
        let text = "demo/demo.txt";
        assert_eq!("demo.txt", last_part(text, ""));
    }
}
