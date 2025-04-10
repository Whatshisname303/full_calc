use std::iter;

use super::executor::{RuntimeError, Value};

// allow for row or column vectors but that seems kind of dumb so maybe clean up for just columns
fn dot(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match (values.get(0), values.get(1)) {
        (Some(val1), Some(val2)) => {
            match (val1, val2) {
                (Value::Matrix(mat1), Value::Matrix(mat2)) => {
                    if mat1.len() == 0 || mat2.len() == 0 {
                        return Ok(Value::Number(0.0));
                    }

                    let val = match (
                        mat1.len() == 1 && mat2.len() == 1, // one row
                        mat1[0].len() == 1 && mat2[0].len() == 1, // one column
                    ) {
                        (true, false) => iter::zip(&mat1[0], &mat2[0])
                            .map(|(num1, num2)| num1 * num2)
                            .sum(),
                        (false, true) => iter::zip(mat1, mat2)
                            .map(|(vec1, vec2)| vec1[0] * vec2[0])
                            .sum(),
                        (true, true) => mat1[0][0] * mat2[0][0],
                        (false, false) => return Err(RuntimeError::BuiltinFuncErr("incompatible dot inputs".to_string())),
                    };

                    Ok(Value::Number(val))
                },
                _ => Err(RuntimeError::BuiltinFuncErr("incompatible dot inputs".to_string())),
            }
        },
        _ => Err(RuntimeError::WrongNumFunctionArgs {
            fname: "dot".to_string(),
            expected: 2,
            got: values.len()
        }),
    }
}

fn cross(values: Vec<Value>) -> Result<Value, RuntimeError> {
    let (val1, val2) = match (values.get(0), values.get(1)) {
        (Some(val1), Some(val2)) => (val1, val2),
        _ => return Err(RuntimeError::WrongNumFunctionArgs{
            fname: "cross".to_string(),
            expected: 2,
            got: values.len(),
        }),
    };

    let (nums1, nums2): (Vec<_>, Vec<_>) = match (val1, val2) {
        (Value::Matrix(mat1), Value::Matrix(mat2)) => (
            mat1.iter().map(|vec| vec[0]).collect(),
            mat2.iter().map(|vec| vec[0]).collect(),
        ),
        _ => return Err(RuntimeError::BuiltinFuncErr("invalid cross inputs".to_string())),
    };

    if nums1.len() != 3 || nums2.len() != 3 {
        return Err(RuntimeError::BuiltinFuncErr("invalid cross inputs".to_string()));
    }

    Ok(Value::Matrix(vec![
        vec![nums1[1] * nums2[2] - nums1[2] * nums2[1]],
        vec![nums1[2] * nums2[0] - nums1[0] * nums2[2]],
        vec![nums1[0] * nums2[1] - nums1[1] * nums2[0]],
    ]))
}

fn unit(values: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut matrix = match values.get(0) {
        Some(Value::Matrix(input)) => input.clone(),
        _ => return Err(RuntimeError::BuiltinFuncErr("unit expects a vector".to_string())),
    };

    if matrix.is_empty() {
        return Err(RuntimeError::BuiltinFuncErr("cannnot take unit of empty vector".to_string()));
    }

    if matrix[0].len() != 1 {
        return Err(RuntimeError::BuiltinFuncErr("unit expects vector, got matrix".to_string()));
    }

    let magnitude = matrix.iter().map(|row| row[0].powi(2)).sum::<f64>().sqrt();

    for row in matrix.iter_mut() {
        row[0] /= magnitude;
    }

    Ok(Value::Matrix(matrix))
}

fn magnitude(values: Vec<Value>) -> Result<Value, RuntimeError> {
    let matrix = match values.get(0) {
        Some(Value::Matrix(input)) => input,
        _ => return Err(RuntimeError::BuiltinFuncErr("magnitude expects a vector".to_string())),
    };

    if matrix.is_empty() {
        return Ok(Value::Number(0.0));
    }

    if matrix[0].len() != 1 {
        return Err(RuntimeError::BuiltinFuncErr("magnitude expects vector, got matrix".to_string()));
    }

    let magnitude = matrix.iter().map(|row| row[0].powi(2)).sum::<f64>().sqrt();

    Ok(Value::Number(magnitude))
}

fn inv(values: Vec<Value>) -> Result<Value, RuntimeError> {
    let matrix = match values.get(0) {
        Some(Value::Matrix(mat)) => mat,
        _ => return Err(RuntimeError::BuiltinFuncErr("inv expects matrix".to_string())),
    };

    let rows = matrix.len();
    let cols = matrix[0].len();

    if rows != cols {
        return Err(RuntimeError::BuiltinFuncErr("inv requires square matrix".to_string()));
    }

    if rows == 1 {
        return Ok(Value::Matrix(vec![vec![1.0/matrix[0][0]]]));
    }

    let det = det_recurse(matrix);

    if det == 0.0 {
        return Err(RuntimeError::BuiltinFuncErr("matrix is not invertible".to_string()));
    }

    let mut adjoint = adjoint(matrix);

    for row in adjoint.iter_mut() {
        for col in row.iter_mut() {
            *col /= det;
        }
    }

    Ok(Value::Matrix(adjoint))
}

fn transpose(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Matrix(mat)) => Ok(Value::Matrix(trans(mat))),
        _ => Err(RuntimeError::BuiltinFuncErr("transpose expects a matrix".to_string())),
    }
}

fn det(values: Vec<Value>) -> Result<Value, RuntimeError> {
    let matrix = match values.get(0) {
        Some(Value::Matrix(input)) => input,
        _ => return Err(RuntimeError::BuiltinFuncErr("det expects a matrix".to_string())),
    };

    if matrix.is_empty() {
        return Ok(Value::Number(0.0))
    }

    let rows = matrix.len();
    let cols = matrix[0].len();

    if rows != cols {
        return Err(RuntimeError::BuiltinFuncErr("det requires equal row and column length".to_string()));
    }

    if rows == 1 {
        return Ok(Value::Number(matrix[0][0]));
    }

    Ok(Value::Number(det_recurse(matrix)))
}

fn rref(_values: Vec<Value>) -> Result<Value, RuntimeError> {
    todo!();
}

fn log(values: Vec<Value>) -> Result<Value, RuntimeError> {
    let num = match values.get(0) {
        Some(Value::Number(num)) => num,
        _ => return Err(RuntimeError::BuiltinFuncErr("log expects a number".to_string())),
    };
    match values.get(1) {
        Some(Value::Number(base)) => Ok(Value::Number(num.log(*base))),
        _ => Ok(Value::Number(num.log10())),
    }
}

fn ln(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.ln())),
        _ => Err(RuntimeError::BuiltinFuncErr("ln expects a number".to_string())),
    }
}

fn sin(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.sin())),
        _ => Err(RuntimeError::BuiltinFuncErr("sin expects 1 number".to_string())),
    }
}

fn cos(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.cos())),
        _ => Err(RuntimeError::BuiltinFuncErr("cos expects 1 number".to_string())),
    }
}

fn tan(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.tan())),
        _ => Err(RuntimeError::BuiltinFuncErr("tan expects 1 number".to_string())),
    }
}

fn asin(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.asin())),
        _ => Err(RuntimeError::BuiltinFuncErr("asin expects 1 number".to_string())),
    }
}

fn acos(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.acos())),
        _ => Err(RuntimeError::BuiltinFuncErr("acos expects 1 number".to_string())),
    }
}

fn atan(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.atan())),
        _ => Err(RuntimeError::BuiltinFuncErr("atan expects 1 number".to_string())),
    }
}

fn rad(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.to_radians())),
        _ => Err(RuntimeError::BuiltinFuncErr("rad expects 1 number".to_string())),
    }
}

fn deg(values: Vec<Value>) -> Result<Value, RuntimeError> {
    match values.get(0) {
        Some(Value::Number(input)) => Ok(Value::Number(input.to_degrees())),
        _ => Err(RuntimeError::BuiltinFuncErr("deg expects 1 number".to_string())),
    }
}

// -- matrix helper methods ---------------------------------------------------

fn det_recurse(mat: &Vec<Vec<f64>>) -> f64 {
    let size = mat.len();

    if size == 2 {
        return mat[0][0] * mat[1][1] - mat[1][0] * mat[0][1];
    }

    let mut sum = 0.0;

    let mut submat = mat.clone();
    let coefficients = submat.pop().unwrap();

    for i in 0..size {
        let mut minor = submat.clone();
        for row in minor.iter_mut() {
            row.remove(i);
        }
        let mut subdet = det_recurse(&minor);
        if (i + size-1) % 2 != 0 {
            subdet = -subdet;
        }
        sum += subdet * coefficients[i];
    }

    sum
}

// gets submatrix excluding row and col 0 indexed
fn submatrix(matrix: &Vec<Vec<f64>>, row: usize, col: usize) -> Vec<Vec<f64>> {
    let mut output = matrix.clone();
    output.remove(row);
    for row in output.iter_mut() {
        row.remove(col);
    }
    output
}

fn adjoint(matrix: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let size = matrix.len();

    if size == 2 {
        return vec![
            vec![matrix[1][1], -matrix[0][1]],
            vec![-matrix[1][0], matrix[0][0]],
        ];
    }

    let mut output = vec![Vec::with_capacity(size); size];

    for row in 0..size {
        for col in 0..size {
            let submat = submatrix(matrix, row, col);
            let mut cofactor = det_recurse(&submat);
            if (row + col) % 2 != 0 {
                cofactor = -cofactor;
            }
            output[row].push(cofactor);
        }
    }

    trans(&output)
}

fn trans(mat: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let rows = mat.len();
    let cols = mat[0].len();

    let mut output: Vec<Vec<f64>> = vec![Vec::with_capacity(rows); cols];

    for i in 0..rows {
        for j in 0..cols {
            output[j].push(mat[i][j]);
        }
    }
    output
}

pub static FUNCTIONS: [(&str, &[&str], fn(Vec<Value>) -> Result<Value, RuntimeError>); 18] = [
    ("std.dot", &["vec1", "vec2"], dot),
    ("std.cross", &["vec1", "vec2"], cross),
    ("std.unit", &["vector"], unit),
    ("std.magnitude", &["vector"], magnitude),
    ("std.inv", &["matrix"], inv),
    ("std.det", &["matrix"], det),
    ("std.transpose", &["matrix"], transpose),
    ("std.rref", &["matrix"], rref),
    ("std.log", &["number", "base"], log),
    ("std.ln", &["number"], ln),
    ("std.sin", &["number"], sin),
    ("std.cos", &["number"], cos),
    ("std.tan", &["number"], tan),
    ("std.asin", &["number"], asin),
    ("std.acos", &["number"], acos),
    ("std.atan", &["number"], atan),
    ("std.rad", &["number"], rad),
    ("std.deg", &["number"], deg),
];
