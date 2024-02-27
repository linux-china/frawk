use pad::{Alignment, PadStr};

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
}