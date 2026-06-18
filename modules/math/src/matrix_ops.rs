use super::matrix::{Matrix, extract_extra_dims, resolve_matrix};
use psy_types::Value;

pub fn matrix_add(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("MATRIX_ADD expects at least 2 arguments".to_string());
    }
    let extra_dims = extract_extra_dims(args, 2)?;
    let a = resolve_matrix(&args[0], extra_dims, "MATRIX_ADD")?;
    let b = resolve_matrix(&args[1], extra_dims, "MATRIX_ADD")?;

    if a.rows != b.rows || a.cols != b.cols {
        return Err("MATRIX_ADD: matrices must have the same dimensions".to_string());
    }

    let data: Vec<f64> = a
        .data
        .iter()
        .zip(b.data.iter())
        .map(|(x, y)| x + y)
        .collect();
    Ok(Matrix {
        rows: a.rows,
        cols: a.cols,
        data,
    }
    .to_value())
}

pub fn matrix_multiply(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("MATRIX_MULTIPLY expects at least 2 arguments".to_string());
    }
    let extra_dims = extract_extra_dims(args, 2)?;
    let a = resolve_matrix(&args[0], extra_dims, "MATRIX_MULTIPLY")?;
    let b = resolve_matrix(&args[1], extra_dims, "MATRIX_MULTIPLY")?;

    if a.cols != b.rows {
        return Err(format!(
            "MATRIX_MULTIPLY: cannot multiply {}x{} by {}x{}",
            a.rows, a.cols, b.rows, b.cols
        ));
    }

    let mut data = vec![0.0; a.rows * b.cols];
    for i in 0..a.rows {
        for j in 0..b.cols {
            let mut sum = 0.0;
            for k in 0..a.cols {
                sum += a.get(i, k) * b.get(k, j);
            }
            data[i * b.cols + j] = sum;
        }
    }

    Ok(Matrix {
        rows: a.rows,
        cols: b.cols,
        data,
    }
    .to_value())
}

pub fn matrix_transpose(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MATRIX_TRANSPOSE expects at least 1 argument".to_string());
    }
    let extra_dims = extract_extra_dims(args, 1)?;
    let a = resolve_matrix(&args[0], extra_dims, "MATRIX_TRANSPOSE")?;

    let mut data = vec![0.0; a.rows * a.cols];
    for i in 0..a.rows {
        for j in 0..a.cols {
            data[j * a.rows + i] = a.get(i, j);
        }
    }

    Ok(Matrix {
        rows: a.cols,
        cols: a.rows,
        data,
    }
    .to_value())
}

/// Runs Gaussian elimination with partial pivoting, returning the
/// upper-triangular result plus the accumulated sign flip from row swaps.
/// Shared by determinant and inverse since both need this core reduction.
fn gaussian_eliminate(mut m: Vec<f64>, n: usize) -> Option<(Vec<f64>, f64)> {
    let mut sign = 1.0;

    for col in 0..n {
        // Partial pivoting: find the row with the largest value in this column.
        let mut pivot_row = col;
        let mut max_val = m[col * n + col].abs();
        for row in (col + 1)..n {
            let val = m[row * n + col].abs();
            if val > max_val {
                max_val = val;
                pivot_row = row;
            }
        }

        if max_val < 1e-12 {
            return None; // Singular matrix.
        }

        if pivot_row != col {
            for c in 0..n {
                m.swap(col * n + c, pivot_row * n + c);
            }
            sign = -sign;
        }

        for row in (col + 1)..n {
            let factor = m[row * n + col] / m[col * n + col];
            for c in col..n {
                m[row * n + c] -= factor * m[col * n + c];
            }
        }
    }

    Some((m, sign))
}

pub fn matrix_determinant(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MATRIX_DETERMINANT expects at least 1 argument".to_string());
    }
    let extra_dims = extract_extra_dims(args, 1)?;
    let a = resolve_matrix(&args[0], extra_dims, "MATRIX_DETERMINANT")?;

    if a.rows != a.cols {
        return Err("MATRIX_DETERMINANT requires a square matrix".to_string());
    }

    let n = a.rows;
    match gaussian_eliminate(a.data.clone(), n) {
        Some((reduced, sign)) => {
            let mut det = sign;
            for i in 0..n {
                det *= reduced[i * n + i];
            }
            Ok(Value::Number(det))
        }
        None => Ok(Value::Number(0.0)), // Singular matrix has determinant 0.
    }
}

pub fn matrix_inverse(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("MATRIX_INVERSE expects at least 1 argument".to_string());
    }
    let extra_dims = extract_extra_dims(args, 1)?;
    let a = resolve_matrix(&args[0], extra_dims, "MATRIX_INVERSE")?;

    if a.rows != a.cols {
        return Err("MATRIX_INVERSE requires a square matrix".to_string());
    }

    let n = a.rows;

    // Build augmented [A | I] matrix, row-major, width 2n.
    let mut aug = vec![0.0; n * 2 * n];
    for r in 0..n {
        for c in 0..n {
            aug[r * 2 * n + c] = a.get(r, c);
        }
        aug[r * 2 * n + (n + r)] = 1.0;
    }

    for col in 0..n {
        let mut pivot_row = col;
        let mut max_val = aug[col * 2 * n + col].abs();
        for row in (col + 1)..n {
            let val = aug[row * 2 * n + col].abs();
            if val > max_val {
                max_val = val;
                pivot_row = row;
            }
        }

        if max_val < 1e-12 {
            return Err("MATRIX_INVERSE: matrix is singular, cannot invert".to_string());
        }

        if pivot_row != col {
            for c in 0..(2 * n) {
                aug.swap(col * 2 * n + c, pivot_row * 2 * n + c);
            }
        }

        let pivot_val = aug[col * 2 * n + col];
        for c in 0..(2 * n) {
            aug[col * 2 * n + c] /= pivot_val;
        }

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row * 2 * n + col];
            for c in 0..(2 * n) {
                aug[row * 2 * n + c] -= factor * aug[col * 2 * n + c];
            }
        }
    }

    let mut data = vec![0.0; n * n];
    for r in 0..n {
        for c in 0..n {
            data[r * n + c] = aug[r * 2 * n + (n + c)];
        }
    }

    Ok(Matrix {
        rows: n,
        cols: n,
        data,
    }
    .to_value())
}
