use std::fmt;

use super::tokens::Token;

const MAX_BINARY_PRECEDENCE: i8 = 3;

#[derive(Debug)]
pub enum SyntaxError {
    NoClosingParen(Vec<Expression>),
    CallNonIdentifier(Token),
    UnexpectedToken(Token),
    ExpectedButGot(Token, Token),
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::NoClosingParen(expressions) => write!(f, "no closing parentheses around {:?}", expressions),
            SyntaxError::CallNonIdentifier(token) => write!(f, "attempting to call {:?} as a function", token),
            SyntaxError::UnexpectedToken(token) => write!(f, "unexpected {:?}", token),
            SyntaxError::ExpectedButGot(t1, t2) => write!(f, "expected '{:?}' but got '{:?}'", t1, t2),
        }
    }
}

impl std::error::Error for SyntaxError {}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Empty,
    Identifier(String),
    Number(String),
    Group(Box<Expression>),
    Matrix(Vec<Vec<Expression>>),
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    FuncCall(String, Vec<Expression>),
}

struct TreeBuilder {
    tokens: Vec<Token>,
    i: usize
}


impl Token {
    fn matches_binary_precedence(&self, precedence: i8) -> bool {
        match (precedence, self) {
            (0, Token::Assign | Token::AltAssign) => true,
            (1, Token::Plus | Token::Minus) => true,
            (2, Token::Mult | Token::Div) => true,
            (3, Token::Pow) => true,
            _ => false,
        }
    }

    fn matches_unary(&self) -> bool {
        match self {
            Token::Minus => true,
            _ => false,
        }
    }
}

impl TreeBuilder {
    fn new(tokens: Vec<Token>) -> TreeBuilder {
        TreeBuilder {tokens, i: 0}
    }

    fn current(&self) -> &Token {
        match self.tokens.get(self.i) {
            Some(token) => token,
            None => &Token::None,
        }
    }

    fn advance(&mut self, n: usize) {
        self.i += n;
    }

    fn take(&mut self) -> &Token {
        self.i += 1;
        match self.tokens.get(self.i - 1) {
            Some(token) => token,
            None => &Token::None,
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        match self.tokens.get(self.i + offset) {
            Some(token) => token,
            None => &Token::None,
        }
    }

    fn peek_back(&self, offset: usize) -> &Token {
        if offset > self.i {
            return &Token::None;
        }
        match self.tokens.get(self.i - offset) {
            Some(token) => token,
            None => &Token::None,
        }
    }

    fn parse(&mut self) -> Result<Expression, SyntaxError> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, precedence: i8) -> Result<Expression, SyntaxError> {
        let mut lhs = match precedence >= MAX_BINARY_PRECEDENCE {
            false => self.parse_binary(precedence + 1),
            true => self.parse_unary(),
        };

        while self.current().matches_binary_precedence(precedence) {
            let op = self.take().clone();
            let rhs = match precedence >= MAX_BINARY_PRECEDENCE {
                false => self.parse_binary(precedence + 1),
                true => self.parse_unary(),
            };
            lhs = Ok(Expression::Binary(Box::new(lhs?), op, Box::new(rhs?)));
        }

        lhs
    }

    fn parse_unary(&mut self) -> Result<Expression, SyntaxError> {
        match self.current().matches_unary() {
            true => {
                let op = self.take().clone();
                let rhs = self.parse_unary()?;
                Ok(Expression::Unary(op, Box::new(rhs)))
            },
            false => self.parse_function_call(),
        }
    }

    fn parse_function_call(&mut self) -> Result<Expression, SyntaxError> {
        match (self.current(), self.peek(1)) {
            (Token::Identifier(fname), Token::OpenParen) => {
                let fname = fname.clone();
                self.advance(2);
                let fargs = self.parse_function_args()?;
                Ok(Expression::FuncCall(fname, fargs))
            },
            _ => self.parse_base(),
        }
    }

    fn parse_function_args(&mut self) -> Result<Vec<Expression>, SyntaxError> {
        let mut args = Vec::new();
        let mut next_arg = self.parse()?;

        // empty means the closing ')' is already consumed
        if let Expression::Empty = next_arg {
            return Ok(args)
        }

        while let Token::Comma = self.current() {
            self.advance(1);
            args.push(next_arg);
            next_arg = self.parse()?;
        }

        match self.take() {
            Token::CloseParen => {
                args.push(next_arg);
                Ok(args)
            },
            _ => Err(SyntaxError::NoClosingParen(args)),
        }
    }

    fn parse_matrix(&mut self) -> Result<Expression, SyntaxError> {
        let mut rows: Vec<Vec<Expression>> = Vec::new();
        let mut current_row: Vec<Expression> = Vec::new();
        let mut next_value = self.parse()?;

        while let Token::Comma | Token::Semicolon = self.current() {
            self.advance(1);
            current_row.push(next_value);

            match *self.peek_back(1) {
                Token::Comma => next_value = self.parse()?,
                Token::Semicolon => {
                    rows.push(current_row);
                    current_row = Vec::new();
                    next_value = self.parse()?;
                },
                _ => panic!("messed up parsing"),
            };
        }

        current_row.push(next_value);
        rows.push(current_row);

        match self.take() {
            Token::CloseBracket => Ok(Expression::Matrix(rows)),
            t => Err(SyntaxError::ExpectedButGot(Token::CloseBracket, t.clone())),
        }
    }

    fn parse_base(&mut self) -> Result<Expression, SyntaxError> {
        match self.take() {
            Token::Identifier(identifier) => Ok(Expression::Identifier(identifier.clone())),
            Token::Number(num) => Ok(Expression::Number(num.clone())),
            Token::Comment(_) => self.parse_base(),
            Token::CloseParen | Token::None => Ok(Expression::Empty),
            Token::OpenParen => {
                let expression = self.parse()?;
                match self.take() {
                    Token::CloseParen => Ok(Expression::Group(Box::new(expression))),
                    _ => Err(SyntaxError::NoClosingParen(vec![expression])),
                }
            },
            Token::OpenBracket => self.parse_matrix(),
            _ => Err(SyntaxError::UnexpectedToken(self.peek_back(1).clone())),
        }
    }
}

pub fn generate_syntax_tree(tokens: Vec<Token>) -> Result<Expression, SyntaxError> {
    TreeBuilder::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokens::*;

    fn e(s: &str) -> Expression {
        let tokens = tokenize(s);
        generate_syntax_tree(tokens).unwrap()
    }

    fn num(s: &str) -> Box<Expression> {
        Box::new(Expression::Number(s.to_string()))
    }
    fn bin(e1: Box<Expression>, op: Token, e2: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Binary(e1, op, e2))
    }
    fn unary(op: Token, e1: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Unary(op, e1))
    }
    fn group(e1: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::Group(e1))
    }
    fn func(fname: &str, args: Vec<Expression>) -> Box<Expression> {
        Box::new(Expression::FuncCall(fname.to_string(), args))
    }

    #[test]
    fn binary_ops() {
        assert_eq!(e("1 + 2"), Expression::Binary(
            num("1"),
            Token::Plus,
            num("2"),
        ));
    }
    #[test]
    fn operator_chaining() {
        assert_eq!(e("1 - 2 * 3 - 4"), Expression::Binary(
            bin(
                num("1"),
                Token::Minus,
                bin(
                    num("2"),
                    Token::Mult,
                    num("3"),
                ),
            ),
            Token::Minus,
            num("4"),
        ))
    }
    #[test]
    fn unary_ops() {
        assert_eq!(
            e("2 * -2 + 3"),
            Expression::Binary(
                bin(
                    num("2"),
                    Token::Mult,
                    unary(Token::Minus, num("2")),
                ),
                Token::Plus,
                num("3"),
            )
        );
    }
    #[test]
    fn using_parentheses() {
        assert_eq!(
            e("2^(-1 * (24))"),
            Expression::Binary(
                num("2"),
                Token::Pow,
                group(bin(
                    unary(Token::Minus, num("1")),
                    Token::Mult,
                    group(num("24")),
                ))
            )
        );
    }
    #[test]
    fn calling_functions() {
        assert_eq!(
            e("max(1, 2+4, 3) + 1"),
            Expression::Binary(
                func("max", vec![
                    *num("1"),
                    *bin(
                        num("2"),
                        Token::Plus,
                        num("4"),
                    ),
                    *num("3"),
                ]),
                Token::Plus,
                num("1"),
            )
        )
    }
    #[test]
    fn matrices() {
        assert_eq!(
            e("[1, 2, 3; 4, 5, 6; 7, 8, 9]"),
            Expression::Matrix(vec![
                vec![*num("1"), *num("2"), *num("3")],
                vec![*num("4"), *num("5"), *num("6")],
                vec![*num("7"), *num("8"), *num("9")],
            ])
        )
    }
}
