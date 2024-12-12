// no parsing implemented for matrices yet

use super::tokens::Token;

const MAX_BINARY_PRECEDENCE: i8 = 3;

pub enum SyntaxError {
    NoClosingParen(Vec<Expression>),
    CallNonIdentifier(Token),
    UnexpectedToken(Token),
}

pub enum Expression {
    Empty,
    Identifier(String),
    Number(String),
    Group(Box<Expression>),
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
        match self.peek(1) {
            Token::OpenParen => {
                self.advance(2);
                let fname = self.peek_back(2).clone();
                let fargs = self.parse_function_args()?;
                match fname {
                    Token::Identifier(id) => Ok(Expression::FuncCall(id, fargs)),
                    _ => Err(SyntaxError::CallNonIdentifier(fname)),
                }
            },
            _ => self.parse_base(),
        }
    }

    fn parse_function_args(&mut self) -> Result<Vec<Expression>, SyntaxError> {
        let mut args = Vec::new();
        let mut next_arg = self.parse()?;

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

    fn parse_base(&mut self) -> Result<Expression, SyntaxError> {
        match self.take() {
            Token::Identifier(identifier) => Ok(Expression::Identifier(identifier.clone())),
            Token::Number(num) => Ok(Expression::Number(num.clone())),
            Token::CloseParen | Token::None => Ok(Expression::Empty),
            Token::OpenParen => {
                let expression = self.parse()?;
                match self.take() {
                    Token::CloseParen => Ok(Expression::Group(Box::new(expression))),
                    _ => Err(SyntaxError::NoClosingParen(vec![expression])),
                }
            },
            _ => Err(SyntaxError::UnexpectedToken(self.peek_back(1).clone())),
        }
    }
}

pub fn generate_syntax_tree(tokens: Vec<Token>) -> Result<Expression, SyntaxError> {
    TreeBuilder::new(tokens).parse()
}
