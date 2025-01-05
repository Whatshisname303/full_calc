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

pub static FUNCTIONS: [(&str, &[&str], fn(Vec<Value>) -> Result<Value, RuntimeError>); 2] = [
    ("std.dot", &["vec1", "vec2"], dot),
    ("std.cross", &["vec1", "vec2"], cross),
];
