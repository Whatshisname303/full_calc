
#[derive(Debug)]
enum TokenType {
    Unknown,
    Identifier,
    Operator,
}

const OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "=", "=>", "(", ")", "^", ","
];

// might want to add type as a struct field for performance
#[derive(Default)]
struct TokenState {
    tokens: Vec<String>,
    current_buffer: String,
}

impl TokenState {
    fn continues_op(&self, ch: char) -> bool {
        let mut joined = self.current_buffer.clone();
        joined.push(ch);
        OPERATORS.iter().any(|op| op.starts_with(&joined))
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
        self.tokens.push(self.current_buffer.clone());
        self.current_buffer.clear();
    }

    fn consume(mut self) -> Vec<String> {
        if !self.current_buffer.is_empty() {
            self.tokens.push(self.current_buffer);
        }
        self.tokens
    }
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '.'
}

pub fn tokenize(line: &str) -> Vec<String> {
    let mut token_state = TokenState::default();

    for ch in line.chars() {
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
    use super::tokenize;

    fn map(vec: &Vec<String>) -> Vec<&str> {
        vec.iter().map(|slice| slice.as_str()).collect()
    }

    #[test]
    fn basic_generation() {
        let t = tokenize("def f(a, b) = a * b");
        assert_eq!(map(&t), vec!["def","f","(","a",",","b",")","=","a","*","b"]);

        let t = tokenize("1 + 1 => hi");
        assert_eq!(map(&t), vec!["1","+","1","=>","hi"]);

    }
    #[test]
    fn nesting() {
        let t = tokenize("2 + ( 4*4 ) = ( 1 + too )");
        assert_eq!(map(&t), vec!["2","+","(","4","*","4",")","=","(","1","+","too",")"]);
    }
    #[test]
    fn whitespace() {
        let t = tokenize("   foryou  = 2*2+pi");
        assert_eq!(map(&t), vec!["foryou","=","2","*","2","+","pi"]);
    }
    #[test]
    fn crunched_ops() {
        let t = tokenize("-(var+-2)^-1=>var2");
        assert_eq!(map(&t), vec!["-","(","var","+","-","2",")","^","-","1","=>","var2"]);
    }
}
