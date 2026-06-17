use pseudocode_types::Value;

/// A resolved matrix: row-major flat data plus its dimensions.
pub(crate) struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>, // row-major: data[r * cols + c]
}

impl Matrix {
    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }

    pub fn to_value(&self) -> Value {
        // Returns as a flat array, matching the existing flat convention
        // your examples already use (A[0]..A[3] for a 2x2).
        Value::Array(self.data.iter().map(|&n| Value::Number(n)).collect())
    }
}

/// Resolves a matrix argument plus optional trailing (rows, cols) args
/// into a validated Matrix. `extra_dims` is whatever numeric arguments
/// followed the array argument in the original call (e.g. for
/// MATRIX_ADD(A, B, rows, cols), each of A and B gets the same extra_dims).
pub(crate) fn resolve_matrix(
    value: &Value,
    extra_dims: Option<(usize, usize)>,
    fn_name: &str,
) -> Result<Matrix, String> {
    match value {
        Value::Array(elements) => {
            // Nested case: array of arrays.
            if elements.iter().all(|e| matches!(e, Value::Array(_))) && !elements.is_empty() {
                let rows = elements.len();
                let mut cols = None;
                let mut data = Vec::with_capacity(rows * rows);

                for row in elements {
                    if let Value::Array(row_elements) = row {
                        if let Some(expected) = cols {
                            if row_elements.len() != expected {
                                return Err(format!(
                                    "{}: jagged matrix, rows have inconsistent length",
                                    fn_name
                                ));
                            }
                        } else {
                            cols = Some(row_elements.len());
                        }

                        for elem in row_elements {
                            match elem {
                                Value::Number(n) => data.push(*n),
                                _ => {
                                    return Err(format!(
                                        "{}: matrix elements must be numbers",
                                        fn_name
                                    ));
                                }
                            }
                        }
                    }
                }

                let cols = cols.unwrap_or(0);
                return Ok(Matrix { rows, cols, data });
            }

            // Flat case: array of plain numbers.
            let mut data = Vec::with_capacity(elements.len());
            for elem in elements {
                match elem {
                    Value::Number(n) => data.push(*n),
                    _ => return Err(format!("{}: matrix elements must be numbers", fn_name)),
                }
            }

            if let Some((rows, cols)) = extra_dims {
                if rows * cols != data.len() {
                    return Err(format!(
                        "{}: {} elements cannot form a {}x{} matrix",
                        fn_name,
                        data.len(),
                        rows,
                        cols
                    ));
                }
                Ok(Matrix { rows, cols, data })
            } else {
                let len = data.len();
                let side = (len as f64).sqrt().round() as usize;
                if side * side != len {
                    return Err(format!(
                        "{}: {} elements is not a perfect square; pass explicit rows/cols for non-square matrices",
                        fn_name, len
                    ));
                }
                Ok(Matrix {
                    rows: side,
                    cols: side,
                    data,
                })
            }
        }
        _ => Err(format!("{} expects an array (matrix) argument", fn_name)),
    }
}

/// Extracts optional trailing (rows, cols) numeric arguments from `args`,
/// starting at `start_index`. Returns None if no trailing numeric args
/// were given at all.
pub(crate) fn extract_extra_dims(
    args: &[Value],
    start_index: usize,
) -> Result<Option<(usize, usize)>, String> {
    if args.len() <= start_index {
        return Ok(None);
    }
    if args.len() != start_index + 2 {
        return Err("expected exactly 2 trailing arguments for rows and cols".to_string());
    }
    let rows = super::expect_number(&args[start_index], "matrix dims")? as usize;
    let cols = super::expect_number(&args[start_index + 1], "matrix dims")? as usize;
    Ok(Some((rows, cols)))
}
