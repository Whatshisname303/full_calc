
use std::{error::Error, fmt, iter};

use crate::{app::state::{Context, FunctionBody}, parser::{highlighting::{HighlightToken, HighlightTokenType}, syntax_tree::Expression, tokens::Token}};

type Num = f64;
type MatrixBody = Vec<Vec<Num>>;

#[derive(Debug, Clone)]
pub enum Value {
    Number(Num),
    Matrix(MatrixBody),
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
    WrongNumFunctionArgs{fname: String, expected: usize, got: usize},
    BuiltinFuncErr(String),
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
            RuntimeError::WrongNumFunctionArgs { fname, expected, got } => write!(f, "{fname} expected {expected} arguments but got {got}"),
            RuntimeError::BuiltinFuncErr(st) => write!(f, "{st}"),
        }
    }
}

impl Error for RuntimeError {}

// might pull out operator implementation and implement on Value enum if convenient in the future
impl Context<'_> {
    pub fn execute(&mut self, expression: Expression) -> Result<Value, RuntimeError> {
        match expression {
            Expression::Number(st) => match st.parse::<Num>() {
                Ok(num) => Ok(Value::Number(num)),
                Err(_) => Err(RuntimeError::BadNumber(st)),
            }
            Expression::Identifier(identifier) => match self.get_var(&identifier) {
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
                match op {
                    Token::Assign => match *lhs {
                        Expression::Identifier(identifier) => {
                            let value = self.execute(*rhs)?;
                            self.set_var(identifier, value.clone());
                            Ok(value)
                        },
                        _ => Err(RuntimeError::AssigningToValue(self.execute(*lhs)?.as_string())),
                    },
                    Token::AltAssign => match *rhs {
                        Expression::Identifier(identifier) => {
                            let value = self.execute(*lhs)?;
                            self.set_var(identifier, value.clone());
                            Ok(value)
                        },
                        _ => Err(RuntimeError::AssigningToValue(self.execute(*rhs)?.as_string())),
                    },
                    _ => {
                        let lval = self.execute(*lhs)?;
                        let rval = self.execute(*rhs)?;
                        lval.binary_op(op, &rval)
                    }
                }
            },
            Expression::FuncCall(fname, args) => {
                let arg_values = args.into_iter()
                    .map(|arg| self.execute(arg))
                    .collect::<Result<Vec<Value>, RuntimeError>>()?;

                let mut function_context = Context::from_context(&self);
                let function_def = self.get_function(&fname).ok_or(RuntimeError::UnknownIdentifier(fname.clone()))?;

                match &function_def.body {
                    FunctionBody::Builtin(closure) => closure(arg_values),
                    FunctionBody::User(body) => {
                        if function_def.params.len() != arg_values.len() {
                            return Err(RuntimeError::WrongNumFunctionArgs {
                                fname,
                                expected: function_def.params.len(),
                                got: arg_values.len(),
                            });
                        }
                        for (param, arg) in iter::zip(&function_def.params, arg_values) {
                            function_context.set_var(param.clone(), arg);
                        }
                        function_context.execute(body.clone())
                    },
                }
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
                        .collect::<Result<Vec<Num>, RuntimeError>>()
                };

                let evaluated_rows = rows.into_iter()
                    .map(map_row)
                    .collect::<Result<MatrixBody, RuntimeError>>()?;

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

// transformations
impl Value {
    pub fn binary_op(&self, op: Token, rhs: &Value) -> Result<Value, RuntimeError> {
        match op {
            Token::Plus => match self {
                Value::Number(num1) => match rhs {
                    Value::Number(num2) => Ok(Value::Number(num1 + num2)),
                    Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("{:?} + {:?}", self, rhs))),
                },
                Value::Matrix(mat1) => match rhs {
                    Value::Number(_) => Err(RuntimeError::InvalidOperation(format!("matrix + {:?}", rhs))),
                    Value::Matrix(mat2) => matrix_matrix_transform_elements(mat1, mat2, |(num1, num2)| num1 + num2),
                },
            },
            Token::Minus => match self {
                Value::Number(num1) => match rhs {
                    Value::Number(num2) => Ok(Value::Number(num1 - num2)),
                    Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("{:?} - {:?}", self, rhs))),
                },
                Value::Matrix(mat1) => match rhs {
                    Value::Number(_) => Err(RuntimeError::InvalidOperation(format!("matrix - {:?}", rhs))),
                    Value::Matrix(mat2) => matrix_matrix_transform_elements(mat1, mat2, |(num1, num2)| num1 - num2),
                },
            },
            Token::Mult => match self {
                Value::Number(num1) => match rhs {
                    Value::Number(num2) => Ok(Value::Number(num1 * num2)),
                    Value::Matrix(mat2) => matrix_transform_elements(mat2, |num| num * num1),
                },
                Value::Matrix(mat1) => match rhs {
                    Value::Number(num2) => matrix_transform_elements(mat1, |num| num * num2),
                    Value::Matrix(mat2) => matrix_multiplication(mat1, mat2),
                },
            },
            Token::Div => match self {
                Value::Number(num1) => match rhs {
                    Value::Number(num2) => Ok(Value::Number(num1 / num2)),
                    Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("{num1} / matrix"))),
                },
                Value::Matrix(_) => match rhs {
                    Value::Number(_) => Err(RuntimeError::InvalidOperation(format!("matrix / number"))),
                    Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("matrix / matrix"))),
                }
            },
            Token::Pow => match self {
                Value::Number(num1) => match rhs {
                    Value::Number(num2) => Ok(Value::Number(num1.powf(*num2))),
                    Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("number ^ matrix"))),
                },
                Value::Matrix(_mat1) => match rhs {
                    Value::Number(_num2) => todo!(),
                    Value::Matrix(_) => Err(RuntimeError::InvalidOperation(format!("matrix ^ matrix"))),
                }
            },
            _ => Err(RuntimeError::ParserFailure("ops got set up weird".into())),
        }
    }
}

fn match_matrices(mat1: &MatrixBody, mat2: &MatrixBody, matcher: impl Fn((usize, usize, usize, usize)) -> bool) -> Result<(), RuntimeError> {
    let m1 = mat1.len();
    let m2 = mat2.len();
    let n1 = mat1.get(0).map(|v| v.len()).unwrap_or(0);
    let n2 = mat2.get(0).map(|v| v.len()).unwrap_or(0);
    match matcher((m1, n1, m2, n2)) {
        true => Ok(()),
        false => Err(RuntimeError::IncompatibleMatrices(m1, n1, m2, n2)),
    }
}

fn matrix_transform_elements(matrix: &MatrixBody, transform: impl Fn(&Num) -> Num) -> Result<Value, RuntimeError> {
    let res = matrix.iter()
        .map(|vec| {
            vec.iter()
                .map(&transform)
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(Value::Matrix(res))
}

fn matrix_matrix_transform_elements(mat1: &MatrixBody, mat2: &MatrixBody, transform: impl Fn((&Num, &Num)) -> Num) -> Result<Value, RuntimeError> {
    match_matrices(mat1, mat2, |(m1, n1, m2, n2)| m1 == m2 && n1 == n2)?;
    let res = iter::zip(mat1, mat2)
        .map(|(vec1, vec2)| {
            iter::zip(vec1, vec2)
                .map(&transform)
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(Value::Matrix(res))
}

fn matrix_multiplication(mat1: &MatrixBody, mat2: &MatrixBody) -> Result<Value, RuntimeError> {
    let m1 =  mat1.len();
    let m2 = mat2.len();
    let n1 = mat1.get(0).map(|r| r.len()).unwrap_or(0);
    let n2 = mat2.get(0).map(|r| r.len()).unwrap_or(0);

    if m1 == 0 || m2 == 0 || n1 != m2 {
        return Err(RuntimeError::IncompatibleMatrices(m1, n1, m2, n2))
    }

    let mut output_rows: MatrixBody = iter::repeat(Vec::new()).take(m1).collect();

    for (row_i, row1) in mat1.iter().enumerate() {
        for col_i in 0..n2 {
            let output_num: Num = mat2.iter()
                .enumerate()
                .map(|(num_i, row2)| row1[num_i] * row2[col_i])
                .sum();
            output_rows[row_i].push(output_num);
        }
    }

    Ok(Value::Matrix(output_rows))
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
