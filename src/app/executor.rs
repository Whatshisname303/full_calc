
use std::{error::Error, fmt, iter};

use crate::parser::{highlighting::{HighlightToken, HighlightTokenType}, syntax_tree::Expression, tokens::Token};
use super::state::App;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Matrix(Vec<Vec<f64>>),
}

#[derive(Debug)]
pub enum RuntimeError {
    BadNumber(String),
    UnknownIdentifier(String),
    ParserFailure(String),
    InvalidOperation(String),
    AssigningToValue(String),
    MatrixUnevenColumns(usize, usize),
    NestedMatrix,
    IncompatibleMatrices(usize, usize, usize, usize),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::BadNumber(st) => write!(f, "bad number: {st}"),
            RuntimeError::UnknownIdentifier(st) => write!(f, "unknown identifier: {st}"),
            RuntimeError::ParserFailure(st) => write!(f, "parser failure: {st}"),
            RuntimeError::InvalidOperation(st) => write!(f, "invalid operation: {st}"),
            RuntimeError::AssigningToValue(st) => write!(f, "attempting to assign to value: {st}"),
            RuntimeError::MatrixUnevenColumns(col1, col2) => write!(f, "matrix columns must be equal length, found {} and {}", col1, col2),
            RuntimeError::NestedMatrix => write!(f, "nested matrices not supported"),
            RuntimeError::IncompatibleMatrices(m1, n1, m2, n2) => write!(f, "cannot multiply {m1}x{n1} with {m2}x{n2}"),
        }
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
            Expression::Identifier(identifier) => match self.context.get_var(&identifier) {
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
                            self.context.set_var(identifier, value.clone());
                            Ok(value)
                        },
                        _ => Err(RuntimeError::AssigningToValue(self.execute(*lhs)?.as_string())),
                    }
                } else if let Token::AltAssign = op {
                    match *rhs {
                        Expression::Identifier(identifier) => {
                            let value = self.execute(*lhs)?;
                            self.context.set_var(identifier, value.clone());
                            Ok(value)
                        },
                        _ => Err(RuntimeError::AssigningToValue(self.execute(*rhs)?.as_string())),
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
                                Value::Matrix(mat2) => {
                                    let m1 =  mat1.len();
                                    let m2 = mat2.len();
                                    let n1 = mat1.get(0).map(|r| r.len()).unwrap_or(0);
                                    let n2 = mat2.get(0).map(|r| r.len()).unwrap_or(0);

                                    if m1 == 0 || m2 == 0 || n1 != m2 {
                                        return Err(RuntimeError::IncompatibleMatrices(m1, n1, m2, n2))
                                    }

                                    let mut output_rows: Vec<Vec<f64>> = iter::repeat(Vec::new()).take(m1).collect();

                                    for (row_i, row1) in mat1.iter().enumerate() {
                                        for col_i in 0..n2 {
                                            let output_num: f64 = mat2.iter()
                                                .enumerate()
                                                .map(|(num_i, row2)| row1[num_i] * row2[col_i])
                                                .sum();
                                            output_rows[row_i].push(output_num);
                                        }
                                    }

                                    Ok(Value::Matrix(output_rows))
                                }
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
            Expression::Matrix(rows) => {
                let map_row = |row: Vec<Expression>| {
                    row.into_iter()
                        .map(|exp| match self.execute(exp) {
                            Ok(val) => match val {
                                Value::Number(num) => Ok(num),
                                Value::Matrix(_) => Err(RuntimeError::NestedMatrix),
                            }
                            Err(e) => Err(e),
                        })
                        .collect::<Result<Vec<f64>, RuntimeError>>()
                };

                let evaluated_rows = rows.into_iter()
                    .map(map_row)
                    .collect::<Result<Vec<Vec<f64>>, RuntimeError>>()?;

                let num_cols = evaluated_rows.get(0).map(|row| row.len()).unwrap_or(0);

                for row in &evaluated_rows {
                    if row.len() != num_cols {
                        return Err(RuntimeError::MatrixUnevenColumns(num_cols, row.len()));
                    }
                }

                Ok(Value::Matrix(evaluated_rows))
            }
            Expression::Group(inner) => self.execute(*inner),
            Expression::Empty => Ok(Value::Number(0.0)), // this might need to be handled different in some cases
        }
    }
}

// displaying values
impl Value {
    pub fn as_string(&self) -> String {
        let mut output = String::new();
        match self {
            Value::Number(num) => output.push_str(&num.to_string()),
            Value::Matrix(rows) => {
                match rows.len() {
                    0 => output.push_str("[Empty]"),
                    _ => {
                        output.push('[');
                        for row in rows {
                            for col in row {
                                output.push_str(&col.to_string());
                                output.push(',');
                                output.push(' ');
                            }
                            output.pop();
                            output.pop();
                            output.push(';');
                            output.push(' ');
                        }
                        output.pop();
                        output.pop();
                        output.push(']');
                    },
                };
            },
        };
        output
    }

    pub fn short_string(&self) -> String {
        match self {
            Value::Number(_) => self.as_string(),
            Value::Matrix(rows) => format!("{}x{}", rows.len(), rows.get(0).map(|r| r.len()).unwrap_or(0)),
        }
    }

    pub fn output_tokens(&self) -> Vec<HighlightToken> {
        match self {
            Value::Number(num) => vec![HighlightToken {text: num.to_string(), kind: HighlightTokenType::Number}],
            Value::Matrix(rows) => {
                let mut tokens = Vec::new();

                if rows.len() == 0 {
                    tokens.push(HighlightToken::op("[Empty]"));

                } else if rows.len() == 1 || rows[0].len() == 1 {
                    tokens.push(HighlightToken::op("["));

                    let (elements, delimiter) = match rows.len() {
                        1 => (&rows[0], ','),
                        _ => (&rows.iter().map(|row| row[0]).collect(), ';'),
                    };

                    for number in elements {
                        tokens.push(HighlightToken::number(number.to_string()));
                        tokens.push(HighlightToken::op(&delimiter.to_string()));
                        tokens.push(HighlightToken::space());
                    }

                    tokens.pop();
                    tokens.pop();

                    tokens.push(HighlightToken::op("]"));

                } else {
                    tokens.push(HighlightToken::op("["));

                    for row in rows {
                        tokens.push(HighlightToken::newline());
                        tokens.push(HighlightToken::tab());
                        for number in row {
                            tokens.push(HighlightToken::number(number.to_string()));
                            tokens.push(HighlightToken::op(", "));
                        }
                        tokens.pop();
                    }

                    tokens.push(HighlightToken::newline());
                    tokens.push(HighlightToken::op("]"))
                }

                tokens
            },
        }
    }
}
