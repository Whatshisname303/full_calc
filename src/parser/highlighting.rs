#[derive(Clone, PartialEq, Debug)]
pub enum HighlightTokenType {
    Identifier,
    Number,
    Operator,
    Command,
    Space,
    Newline,
    Tab,
}

#[derive(PartialEq, Debug)]
pub struct HighlightToken {
    pub text: String,
    pub kind: HighlightTokenType,
}

pub fn get_highlight_tokens(line: &str) -> Vec<HighlightToken> {
    Tokenizer::new(line).consume()
}

struct Tokenizer<'a> {
    line: &'a str,
    tokens: Vec<HighlightToken>,
    current_kind: HighlightTokenType,
    current_buf: String,
}

impl Tokenizer<'_> {
    fn new(line: &str) -> Tokenizer {
        Tokenizer {
            line,
            tokens: Vec::new(),
            current_kind: HighlightTokenType::Space,
            current_buf: String::new(),
        }
    }

    fn consume(mut self) -> Vec<HighlightToken> {
        for ch in self.line.chars() {
            if ch == '\r' || ch == '\n' {
                self.flush_token();
                self.tokens.push(HighlightToken {
                    text: String::new(),
                    kind: HighlightTokenType::Newline,
                });
                continue;
            }

            if ch == '\t' {
                self.flush_token();
                self.tokens.push(HighlightToken {
                    text: String::new(),
                    kind: HighlightTokenType::Tab,
                });
                continue;
            }

            match self.current_kind.clone() {
                HighlightTokenType::Space => {
                    if ch == ' ' {
                        self.current_buf.push(ch);
                    } else {
                        self.start_token(ch);
                    }
                },
                HighlightTokenType::Identifier | HighlightTokenType::Number => {
                    if ch.is_alphanumeric() || ch == '.' || ch == '_' {
                        self.current_buf.push(ch);
                    } else {
                        self.start_token(ch);
                    }
                },
                HighlightTokenType::Operator => {
                    if ch.is_alphanumeric() || ch == '.' || ch == '_' {
                        self.start_token(ch);
                    } else {
                        self.current_buf.push(ch)
                    }
                },
                HighlightTokenType::Command |
                HighlightTokenType::Newline |
                HighlightTokenType::Tab => panic!("token type set weird"),
            }
        }

        self.flush_token();
        self.tokens
    }

    // true for all primary commands, true for command params if prior token is command
    fn current_is_command(&self) -> bool {
        match self.current_buf.as_str() {
            "clear" | "quit" | "exit" | "reload" | "use" | "load" | "def" | "config" | "show" | "panel" => true,
            "raw" | "deg" | "rad" | "vars" | "autocomplete" | "preview" => {
                match self.tokens.iter().rev().find(|token| token.kind != HighlightTokenType::Space) {
                    Some(token) => match token.kind {
                        HighlightTokenType::Command => true,
                        _ => false,
                    },
                    None => false,
                }
            },
            _ => false,
        }
    }

    fn flush_token(&mut self) {
        if self.current_buf.is_empty() {
            return;
        }

        if self.current_is_command() {
            self.current_kind = HighlightTokenType::Command;
        }

        self.tokens.push(HighlightToken {
            text: self.current_buf.clone(),
            kind: self.current_kind.clone(),
        });

        self.current_buf.clear();
    }

    fn start_token(&mut self, ch: char) {
        self.flush_token();
        self.current_buf.push(ch);

        if ch.is_whitespace() {
            self.current_kind = HighlightTokenType::Space;
        } else if ch.is_alphabetic() {
            self.current_kind = HighlightTokenType::Identifier;
        } else if ch.is_numeric() || ch == '.' {
            self.current_kind = HighlightTokenType::Number;
        } else {
            self.current_kind = HighlightTokenType::Operator;
        }
    }
}

impl HighlightToken {
    pub fn text(text: String) -> HighlightToken {
        HighlightToken {
            text,
            kind: HighlightTokenType::Identifier,
        }
    }
    pub fn number(text: String) -> HighlightToken {
        HighlightToken {
            text,
            kind: HighlightTokenType::Number,
        }
    }
    pub fn op(text: &str) -> HighlightToken {
        HighlightToken {
            text: text.to_string(),
            kind: HighlightTokenType::Operator,
        }
    }
    pub fn newline() -> HighlightToken {
        HighlightToken {
            text: String::new(),
            kind: HighlightTokenType::Newline,
        }
    }
    pub fn tab() -> HighlightToken {
        HighlightToken {
            text: String::new(),
            kind: HighlightTokenType::Tab,
        }
    }
    pub fn space() -> HighlightToken {
        HighlightToken {
            text: String::from(' '),
            kind: HighlightTokenType::Space,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_highlighting() {
        let t = get_highlight_tokens("hello clear 50.2 =>");
        assert_eq!(t, vec![
            HighlightToken {text: "hello".to_string(), kind: HighlightTokenType::Identifier},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "clear".to_string(), kind: HighlightTokenType::Command},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "50.2".to_string(), kind: HighlightTokenType::Number},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "=>".to_string(), kind: HighlightTokenType::Operator},
        ]);
    }

    #[test]
    fn token_splitting() {
        let t = get_highlight_tokens("50.2 +hi.me=>clear");
        assert_eq!(t, vec![
            HighlightToken {text: "50.2".to_string(), kind: HighlightTokenType::Number},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "+".to_string(), kind: HighlightTokenType::Operator},
            HighlightToken {text: "hi.me".to_string(), kind: HighlightTokenType::Identifier},
            HighlightToken {text: "=>".to_string(), kind: HighlightTokenType::Operator},
            HighlightToken {text: "clear".to_string(), kind: HighlightTokenType::Command},
        ]);
    }

    #[test]
    fn commands() {
        let t = get_highlight_tokens("raw clear hi deg panel vars");
        assert_eq!(t, vec![
            HighlightToken {text: "raw".to_string(), kind: HighlightTokenType::Identifier},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "clear".to_string(), kind: HighlightTokenType::Command},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "hi".to_string(), kind: HighlightTokenType::Identifier},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "deg".to_string(), kind: HighlightTokenType::Identifier},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "panel".to_string(), kind: HighlightTokenType::Command},
            HighlightToken {text: " ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "vars".to_string(), kind: HighlightTokenType::Command},
        ]);
    }

    #[test]
    fn whitespace() {
        let t = get_highlight_tokens("1+.5\n -2.0");
        assert_eq!(t, vec![
            HighlightToken {text: "1".to_string(), kind: HighlightTokenType::Number},
            HighlightToken {text: "+".to_string(), kind: HighlightTokenType::Operator},
            HighlightToken {text: ".5".to_string(), kind: HighlightTokenType::Number},
            HighlightToken {text: "\n ".to_string(), kind: HighlightTokenType::Space},
            HighlightToken {text: "-".to_string(), kind: HighlightTokenType::Operator},
            HighlightToken {text: "2.0".to_string(), kind: HighlightTokenType::Number},
        ]);
    }
}
