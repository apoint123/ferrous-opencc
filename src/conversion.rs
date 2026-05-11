use std::sync::Arc;

use crate::dictionary::Dictionary;

pub struct Converter {
    dict: Arc<dyn Dictionary>,
}

impl Converter {
    pub fn new(dict: Arc<dyn Dictionary>) -> Self {
        Self { dict }
    }

    pub fn convert(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() + text.len() / 4);
        let mut i = 0;

        while i < text.len() {
            let remaining_text = &text[i..];

            if let Some((key, value)) = self.dict.match_prefix(remaining_text) {
                result.push_str(value);
                i += key.len();
            } else {
                let ch = remaining_text.chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        }

        result
    }
}
