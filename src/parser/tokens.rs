use std::fmt;

#[derive(Debug)]
enum TokenType {
    Unknown,
    Identifier,
    Operator,
}

static OPERATORS: &[(&str, Token)] = &[
    ("(", Token::OpenParen),
    (")", Token::CloseParen),
    ("[", Token::OpenBracket),
    ("]", Token::CloseBracket),
    ("^", Token::Pow),
    ("*", Token::Mult),
    ("/", Token::Div),
    ("+", Token::Plus),
    ("-", Token::Minus),
    ("=", Token::Assign),
    ("=>", Token::AltAssign),
    (",", Token::Comma),
    (":", Token::Colon),
    (";", Token::Semicolon),
];

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Identifier(String),
    Number(String),
    Comment(String),
    Unknown(String),
    None,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Pow,
    Mult,
    Div,
    Plus,
    Minus,
    Assign,
    AltAssign,
    Comma,
    Colon,
    Semicolon,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "{s}"),
            Token::Number(s) => write!(f, "{s}"),
            Token::Comment(s) => write!(f, "{s}"),
            Token::Unknown(s) => write!(f, "{s}"),
            Token::None => write!(f, ""),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::Pow => write!(f, "^"),
            Token::Mult => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Assign => write!(f, "="),
            Token::AltAssign => write!(f, "=>"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
        }
    }
}

impl Token {
    pub fn is_binary_op(&self) -> bool {
        match *self {
            Token::Plus | Token::Minus |
            Token::Mult | Token::Div |
            Token::Pow |
            Token::Assign | Token::AltAssign => true,
            _ => false,
        }
    }

    pub fn is_from_str(&self, st: &str) -> bool {
        match *self {
            Token::Identifier(ref ident) => ident == st,
            Token::Number(_) => false,
            ref token => OPERATORS.iter().any(|(op_str, op_token)| *op_str == st && op_token == token),
        }
    }
}

// might want to add type as a struct field so not constantly recomputing
#[derive(Default)]
struct TokenState {
    tokens: Vec<Token>,
    current_buffer: String,
}

impl TokenState {
    fn continues_op(&self, ch: char) -> bool {
        let mut joined = self.current_buffer.clone();
        joined.push(ch);
        OPERATORS.iter().any(|(op, _)| op.starts_with(&joined))
    }

    fn get_type(&self) -> TokenType {
        if self.current_buffer.is_empty() {
            return TokenType::Unknown;
        } else if is_ident(self.current_buffer.chars().next_back().unwrap()) {
            return TokenType::Identifier;
        } else {
            return TokenType::Operator;
        }
    }

    fn flush_token(&mut self) {
        match self.get_type() {
            TokenType::Identifier => {
                if self.current_buffer.chars().next().unwrap().is_alphabetic() {
                    self.tokens.push(Token::Identifier(self.current_buffer.clone()));
                } else {
                    self.tokens.push(Token::Number(self.current_buffer.clone()));
                }
                self.current_buffer.clear();
            },
            TokenType::Operator => {
                let op_type = OPERATORS.iter().find_map(|(st, enum_type)| {
                    match *st == self.current_buffer {
                        true => Some(enum_type),
                        false => None,
                    }
                });

                if let Some(op_type) = op_type {
                    self.tokens.push(op_type.clone());
                    self.current_buffer.clear();
                } else {
                    self.tokens.push(Token::Unknown(std::mem::take(&mut self.current_buffer)));
                }
            },
            TokenType::Unknown => {},
        };
    }

    fn consume(mut self) -> Vec<Token> {
        if !self.current_buffer.is_empty() {
            self.flush_token();
        }
        self.tokens
    }
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '.'
}

pub fn tokenize(line: &str) -> Vec<Token> {
    let mut token_state = TokenState::default();
    let chars: Vec<_> = line.chars().collect();

    for (i, ch) in line.char_indices() {
        if let ('-', Some(&'-')) = (ch, chars.get(i+1)) {
            token_state.flush_token();
            token_state.tokens.push(Token::Comment(String::from(&line[i..])));
            return token_state.consume();
        }

        match token_state.get_type() {
            TokenType::Identifier => {
                if ch.is_whitespace() {
                    token_state.flush_token();
                } else if is_ident(ch) {
                    token_state.current_buffer.push(ch);
                } else {
                    token_state.flush_token();
                    token_state.current_buffer.push(ch);
                }
            },
            TokenType::Operator => {
                if ch.is_whitespace() {
                    token_state.flush_token();
                } else if is_ident(ch) {
                    token_state.flush_token();
                    token_state.current_buffer.push(ch);
                } else if token_state.continues_op(ch) {
                    token_state.current_buffer.push(ch);

                } else {
                    token_state.flush_token();
                    token_state.current_buffer.push(ch);
                }
            },
            TokenType::Unknown => {
                if !ch.is_whitespace() {
                    token_state.current_buffer.push(ch);
                }
            },
        };
    }

    token_state.consume()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_generation() {
        let t = tokenize("hello there");
        assert_eq!(t, vec![
            Token::Identifier("hello".to_string()),
            Token::Identifier("there".to_string()),
        ]);
    }
    #[test]
    fn literals() {
        let t = tokenize(".123+hi-var2");
        assert_eq!(t, vec![
            Token::Number(".123".to_string()),
            Token::Plus,
            Token::Identifier("hi".to_string()),
            Token::Minus,
            Token::Identifier("var2".to_string()),
        ]);
    }
    #[test]
    fn operator_mashing() {
        let t = tokenize("3^-1=>-a");
        assert_eq!(t, vec![
            Token::Number("3".to_string()),
            Token::Pow,
            Token::Minus,
            Token::Number("1".to_string()),
            Token::AltAssign,
            Token::Minus,
            Token::Identifier("a".to_string()),
        ]);
    }
}
