
use std::{error::Error, fmt};

use crate::parser::{syntax_tree::Expression, tokens::Token};
use super::state::App;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Matrix(Vec<Vec<f64>>),
}

impl Value {
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Number(num) => num.to_string(),
            Value::Matrix(_matrix) => {
                todo!();
            }
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    BadNumber(String),
    UnknownIdentifier(String),
    ParserFailure(String),
    InvalidOperation(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}

impl Error for RuntimeError {}

// might pull out operator implementation and implement on Value enum if convenient in the future
impl App {
    pub fn execute(&mut self, expression: Expression) -> Result<Value, RuntimeError> {
        match expression {
            Expression::Number(st) => match st.parse::<f64>() {
                Ok(num) => Ok(Value::Number(num)),
                Err(_) => Err(RuntimeError::BadNumber(st)),
            }
            Expression::Identifier(identifier) => match self.vars.get(&identifier) {
                Some(value) => Ok(value.clone()),
                None => Err(RuntimeError::UnknownIdentifier(identifier.clone()))
            },
            Expression::Unary(op, input) => match op {
                Token::Minus => match self.execute(*input)? {
                    Value::Number(num) => Ok(Value::Number(-num)),
                    Value::Matrix(mut mat) => {
                        for vec in mat.iter_mut() {
                            for num in vec {
                                *num = - *num;
                            }
                        }
                        Ok(Value::Matrix(mat))
                    }
                },
                _ => Err(RuntimeError::ParserFailure(format!("{:?} of {:?}", op, input)))
            },
            Expression::Binary(lhs, op, rhs) => {
                let lhs = self.execute(*lhs)?;
                let rhs = self.execute(*rhs)?;
                match op {
                    Token::Plus => match lhs {
                        Value::Number(num1) => match rhs {
                            Value::Number(num2) => Ok(Value::Number(num1 + num2)),
                            Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("{:?} + {:?}", lhs, rhs))),
                        },
                        Value::Matrix(mat1) => match rhs {
                            Value::Number(num2) => todo!(),
                            Value::Matrix(mat2) => todo!(),
                        },
                    },
                    Token::Minus => todo!(),
                    Token::Mult => todo!(),
                    Token::Div => todo!(),
                    Token::Pow => todo!(),
                    Token::Assign => todo!(),
                    Token::AltAssign => todo!(),
                    _ => Err(RuntimeError::ParserFailure(format!("{:?} with {:?} and {:?}", op, lhs, rhs))),
                }
            },
            Expression::FuncCall(_fname, _args) => {
                todo!();
            },
            Expression::Group(inner) => self.execute(*inner),
            Expression::Empty => Ok(Value::Number(0.0)), // this might need to be handled different in some cases
        }
    }
}
