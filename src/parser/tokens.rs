// I have no clue if this shit works at all
// deleted the tests and haven't been willing to rewrite the utility functions

use std:: error::Error;

#[derive(Debug)]
enum TokenType {
    Unknown,
    Identifier,
    Operator,
}

static OPERATORS: &[(&str, Token, u8)] = &[
    ("(", Token::OpenParen, 1),
    (")", Token::CloseParen, 1),
    ("[", Token::OpenBracket, 1),
    ("]", Token::CloseBracket, 1),
    ("^", Token::Pow, 2),
    ("*", Token::Mult, 2),
    ("/", Token::Div, 2),
    ("+", Token::Plus, 3),
    ("-", Token::Minus, 3),
    ("=", Token::Assign, 4),
    ("=>", Token::AltAssign, 4),
    (",", Token::Comma, 9),
    (":", Token::Colon, 9),
    (";", Token::Semicolon, 9),
];

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Identifier(String),
    Number(String),
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
        OPERATORS.iter().any(|(op, _, _)| op.starts_with(&joined))
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

    fn flush_token(&mut self) -> Result<(), Box<dyn Error>> {
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
                let op_type = OPERATORS.iter().find_map(|(st, enum_type, _)| {
                    match *st == self.current_buffer {
                        true => Some(enum_type),
                        false => None,
                    }
                });

                if let Some(op_type) = op_type {
                    self.tokens.push(op_type.clone());
                    self.current_buffer.clear();
                } else {
                    return Err("unknown operator: (fuck you)".into());
                    // will be more polite in the future
                }
            },
            TokenType::Unknown => {
                return Err("how did I get here...".into())
            },
        };

        Ok(())
    }

    fn consume(mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        if !self.current_buffer.is_empty() {
            self.flush_token()?;
        }
        Ok(self.tokens)
    }
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '.'
}

pub fn tokenize(line: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut token_state = TokenState::default();

    for ch in line.chars() {
        match token_state.get_type() {
            TokenType::Identifier => {
                if ch.is_whitespace() {
                    token_state.flush_token()?;
                } else if is_ident(ch) {
                    token_state.current_buffer.push(ch);
                } else {
                    token_state.flush_token()?;
                    token_state.current_buffer.push(ch);
                }
            },
            TokenType::Operator => {
                if ch.is_whitespace() {
                    token_state.flush_token()?;
                } else if is_ident(ch) {
                    token_state.flush_token()?;
                    token_state.current_buffer.push(ch);
                } else if token_state.continues_op(ch) {
                    token_state.current_buffer.push(ch);

                } else {
                    token_state.flush_token()?;
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
        let t = tokenize("hello there").unwrap();
        assert_eq!(t, vec![
            Token::Identifier("hello".to_string()),
            Token::Identifier("there".to_string()),
        ]);
    }
    #[test]
    fn literals() {
        let t = tokenize(".123+hi-var2").unwrap();
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
        let t = tokenize("3^-1=>-a").unwrap();
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
    #[test]
    fn invalid_operator() {
        let t = vec![
            tokenize("?"),
            tokenize("a <= 1"),
            tokenize("#hi"),
        ];
        t.iter().for_each(|t| assert!(t.is_err()));
    }
}
