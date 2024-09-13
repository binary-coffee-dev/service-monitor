pub trait ToMarkdown {
    fn parse_text_to_markdown(&self) -> String;
}

impl ToMarkdown for String {
    fn parse_text_to_markdown(&self) -> String {
        let mut new_text = String::new();
        for c in self.chars() {
            match c {
                '_' => new_text.push_str("\\_"),
                '*' => new_text.push_str("\\*"),
                '[' => new_text.push_str("\\["),
                ']' => new_text.push_str("\\]"),
                '(' => new_text.push_str("\\("),
                ')' => new_text.push_str("\\)"),
                '~' => new_text.push_str("\\~"),
                '`' => new_text.push_str("\\`"),
                '>' => new_text.push_str("\\>"),
                '#' => new_text.push_str("\\#"),
                '+' => new_text.push_str("\\+"),
                '-' => new_text.push_str("\\-"),
                '=' => new_text.push_str("\\="),
                '|' => new_text.push_str("\\|"),
                '{' => new_text.push_str("\\{"),
                '}' => new_text.push_str("\\}"),
                '.' => new_text.push_str("\\."),
                '!' => new_text.push_str("\\!"),
                _ => new_text.push(c),
            }
        }
        new_text
    }
}
