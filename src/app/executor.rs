
use std::{error::Error, fmt, iter};

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
    AssigningToValue(String),
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
                if let Token::Assign = op {
                    match *lhs {
                        Expression::Identifier(identifier) => {
                            let value = self.execute(*rhs)?;
                            self.vars.insert(identifier, value.clone());
                            Ok(value)
                        },
                        _ => Err(RuntimeError::AssigningToValue(self.execute(*rhs)?.to_display_string())),
                    }
                } else if let Token::AltAssign = op {
                    match *rhs {
                        Expression::Identifier(identifier) => {
                            let value = self.execute(*lhs)?;
                            self.vars.insert(identifier, value.clone());
                            Ok(value)
                        },
                        _ => Err(RuntimeError::AssigningToValue(self.execute(*rhs)?.to_display_string())),
                    }
                } else {
                    let lval = self.execute(*lhs)?;
                    let rval = self.execute(*rhs)?;
                    match op {
                        Token::Plus => match lval {
                            Value::Number(num1) => match rval {
                                Value::Number(num2) => Ok(Value::Number(num1 + num2)),
                                Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("{:?} + {:?}", lval, rval))),
                            },
                            Value::Matrix(mut mat1) => match rval {
                                Value::Number(_) => Err(RuntimeError::InvalidOperation(format!("matrix + {:?}", rval))),
                                Value::Matrix(ref mat2) => {
                                    for (vec1, vec2) in iter::zip(mat1.iter_mut(), mat2) {
                                        for (num1, num2) in iter::zip(vec1, vec2) {
                                            *num1 += num2;
                                        }
                                    }
                                    Ok(Value::Matrix(mat1))
                                },
                            },
                        },
                        Token::Minus => match lval {
                            Value::Number(num1) => match rval {
                                Value::Number(num2) => Ok(Value::Number(num1 - num2)),
                                Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("{:?} - {:?}", lval, rval))),
                            },
                            Value::Matrix(mut mat1) => match rval {
                                Value::Number(_) => Err(RuntimeError::InvalidOperation(format!("matrix - {:?}", rval))),
                                Value::Matrix(ref mat2) => {
                                    for (vec1, vec2) in iter::zip(mat1.iter_mut(), mat2) {
                                        for (num1, num2) in iter::zip(vec1, vec2) {
                                            *num1 -= num2;
                                        }
                                    }
                                    Ok(Value::Matrix(mat1))
                                },
                            },
                        },
                        Token::Mult => match lval {
                            Value::Number(num1) => match rval {
                                Value::Number(num2) => Ok(Value::Number(num1 * num2)),
                                Value::Matrix(mut mat2) => {
                                    for vec in mat2.iter_mut() {
                                        for num in vec {
                                            *num *= num1;
                                        }
                                    }
                                    Ok(Value::Matrix(mat2))
                                },
                            },
                            Value::Matrix(mut mat1) => match rval {
                                Value::Number(num2) => {
                                    for vec in mat1.iter_mut() {
                                        for num in vec {
                                            *num *= num2;
                                        }
                                    }
                                    Ok(Value::Matrix(mat1))
                                },
                                Value::Matrix(_mat2) => todo!(),
                            },
                        },
                        Token::Div => todo!(),
                        Token::Pow => todo!(),
                        _ => Err(RuntimeError::ParserFailure("ops got set up weird".into())),
                    }
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
