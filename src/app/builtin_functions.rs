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

fn inv(_values: Vec<Value>) -> Result<Value, RuntimeError> {
    todo!();
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

    todo!();
}

fn transpose(_values: Vec<Value>) -> Result<Value, RuntimeError> {
    todo!();
}

fn rref(_values: Vec<Value>) -> Result<Value, RuntimeError> {
    todo!();
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

pub static FUNCTIONS: [(&str, &[&str], fn(Vec<Value>) -> Result<Value, RuntimeError>); 16] = [
    ("std.dot", &["vec1", "vec2"], dot),
    ("std.cross", &["vec1", "vec2"], cross),
    ("std.unit", &["vector"], unit),
    ("std.magnitude", &["vector"], magnitude),
    ("std.inv", &["matrix"], inv),
    ("std.det", &["matrix"], det),
    ("std.transpose", &["matrix"], transpose),
    ("std.rref", &["matrix"], rref),
    ("std.sin", &["number"], sin),
    ("std.cos", &["number"], cos),
    ("std.tan", &["number"], tan),
    ("std.asin", &["number"], asin),
    ("std.acos", &["number"], acos),
    ("std.atan", &["number"], atan),
    ("std.rad", &["number"], rad),
    ("std.deg", &["number"], deg),
];
