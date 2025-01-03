use core::fmt;

use crate::app::state::{FunctionBody, FunctionDef};
use super::{syntax_tree::{generate_syntax_tree, SyntaxError}, tokens::Token};

#[derive(Debug)]
pub enum FunctionDefinitionError {
    InvalidName(Token),
    MissingParams,
    MissingAssignOp,
    MissingClosingParen,
    SyntaxError(SyntaxError),
    Default,
}

pub fn parse_function_definition(tokens: &Vec<Token>) -> Result<FunctionDef, FunctionDefinitionError> {
    let function_name = match tokens.get(1) {
        Some(token) => match token {
            Token::Identifier(ident) => ident.clone(),
            _ => return Err(FunctionDefinitionError::InvalidName(token.clone())),
        },
        None => return Err(FunctionDefinitionError::Default),
    };

    if !tokens.get(2).is_some_and(|token| *token == Token::OpenParen) {
        return Err(FunctionDefinitionError::MissingParams);
    }

    let params_start = 3;
    let params_end = tokens.iter().position(|t| *t == Token::CloseParen).ok_or(FunctionDefinitionError::MissingClosingParen)?;

    let mut function_params: Vec<String> = Vec::new();

    for token in &tokens[params_start..params_end] {
        if let Token::Identifier(ident) = token {
            function_params.push(ident.clone())
        }
    }

    if !tokens.get(params_end + 1).is_some_and(|token| *token == Token::Assign) {
        return Err(FunctionDefinitionError::MissingAssignOp);
    }

    let body_tokens = tokens[params_end + 2..].to_vec();
    let function_body = generate_syntax_tree(body_tokens).map_err(|e| FunctionDefinitionError::SyntaxError(e))?;

    Ok(FunctionDef {
        name: function_name,
        params: function_params,
        body: FunctionBody::User(function_body),
    })
}

impl fmt::Display for FunctionDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let usage_example = "usage: def <fname> (<params>) = <body>";
        match self {
            FunctionDefinitionError::InvalidName(token) => write!(f, "expected function name, got {:?}", token),
            FunctionDefinitionError::MissingParams => write!(f, "{usage_example}"),
            FunctionDefinitionError::MissingAssignOp => write!(f, "missing '=' in function definition"),
            FunctionDefinitionError::MissingClosingParen => write!(f, "missing closing ')' for function arguments"),
            FunctionDefinitionError::SyntaxError(e) => write!(f, "syntax error in function body: {e}"),
            FunctionDefinitionError::Default => write!(f, "{usage_example}"),
        }
    }
}

impl std::error::Error for FunctionDefinitionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokens::tokenize;
    use crate::app::state::{FunctionDef, FunctionBody};

    fn e(s: &str) -> Result<FunctionDef, FunctionDefinitionError> {
        parse_function_definition(&tokenize(s).unwrap())
    }

    #[test]
    fn simple_function() {
        let f = e("def add(a, b) = a + b").unwrap();
        assert_eq!(&f.name, "add");
        assert_eq!(f.params, vec!["a".to_string(), "b".to_string()]);
    }
    #[test]
    fn bad_definition() {
        let f = e("def +(a) = a");
        assert!(f.is_err());
    }
    #[test]
    fn simple_definition() {
        let f = e("def a() = 12").unwrap();
        assert_eq!(&f.name, "a");
        let p: Vec<String> = Vec::new();
        assert_eq!(f.params, p);
    }
}
