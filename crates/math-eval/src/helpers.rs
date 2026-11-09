fn range_values(start: f64, end: f64, step: f64) -> Result<Vec<f64>, EvalError> {
    if !start.is_finite() || !end.is_finite() || !step.is_finite() || step == 0.0 {
        return Err(EvalError::Domain(
            "range expects finite values and a non-zero step".into(),
        ));
    }
    let mut values = Vec::new();
    let mut value = start;
    if step > 0.0 {
        while value <= end + step.abs() * 1e-9 {
            values.push(value);
            if values.len() > 100_000 {
                return Err(EvalError::InvalidLoop(
                    "range has more than 100000 elements".into(),
                ));
            }
            value += step;
        }
    } else {
        while value >= end - step.abs() * 1e-9 {
            values.push(value);
            if values.len() > 100_000 {
                return Err(EvalError::InvalidLoop(
                    "range has more than 100000 elements".into(),
                ));
            }
            value += step;
        }
    }
    Ok(values)
}

fn filled_matrix(rows: usize, columns: usize, value: f64) -> Result<Vec<Vec<f64>>, EvalError> {
    if rows == 0 || columns == 0 || rows.saturating_mul(columns) > 1_000_000 {
        return Err(EvalError::InvalidMatrix(
            "matrix dimensions must be between 1x1 and 1x1000000 elements".into(),
        ));
    }
    Ok(vec![vec![value; columns]; rows])
}

fn resize_matrix(
    matrix: &[Vec<f64>],
    rows: usize,
    columns: usize,
) -> Result<Vec<Vec<f64>>, EvalError> {
    let (old_rows, old_columns) = matrix_shape(matrix)?;
    if rows == 0 || columns == 0 || rows.saturating_mul(columns) > 1_000_000 {
        return Err(EvalError::InvalidMatrix(
            "matrix dimensions must be between 1x1 and 1x1000000 elements".into(),
        ));
    }
    let mut result = vec![vec![0.0; columns]; rows];
    for row in 0..rows.min(old_rows) {
        for column in 0..columns.min(old_columns) {
            result[row][column] = matrix[row][column];
        }
    }
    Ok(result)
}

fn reshape_matrix(
    matrix: &[Vec<f64>],
    rows: usize,
    columns: usize,
) -> Result<Vec<Vec<f64>>, EvalError> {
    let (old_rows, old_columns) = matrix_shape(matrix)?;
    if rows == 0
        || columns == 0
        || rows.saturating_mul(columns) != old_rows.saturating_mul(old_columns)
    {
        return Err(EvalError::InvalidMatrix(
            "reshape must preserve the number of elements".into(),
        ));
    }
    let flattened = matrix.iter().flatten().copied().collect::<Vec<_>>();
    Ok((0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| flattened[row * columns + column])
                .collect()
        })
        .collect())
}

fn find_rows(matrix: &[Vec<f64>], value: f64) -> Result<Vec<usize>, EvalError> {
    let (rows, _) = matrix_shape(matrix)?;
    let mut result = Vec::new();
    for row in 0..rows {
        if matrix[row]
            .iter()
            .any(|candidate| (*candidate - value).abs() < 1e-9)
        {
            result.push(row + 1);
        }
    }
    Ok(result)
}

fn sorted_values(matrix: &[Vec<f64>]) -> Result<Vec<f64>, EvalError> {
    matrix_shape(matrix)?;
    let mut values = matrix.iter().flatten().copied().collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    Ok(values)
}

fn sort_rows(matrix: &[Vec<f64>], column: usize) -> Result<Vec<Vec<f64>>, EvalError> {
    let (_, columns) = matrix_shape(matrix)?;
    if column == 0 || column > columns {
        return Err(EvalError::IndexOutOfBounds(format!(
            "column {column} is outside 1..={columns}"
        )));
    }
    let mut rows = matrix.to_vec();
    rows.sort_by(|left, right| left[column - 1].total_cmp(&right[column - 1]));
    Ok(rows)
}

fn vector_values(matrix: &[Vec<f64>]) -> Result<Vec<f64>, EvalError> {
    matrix_shape(matrix)?;
    Ok(matrix.iter().flatten().copied().collect())
}

fn vector_dot(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<f64, EvalError> {
    let left = vector_values(left)?;
    let right = vector_values(right)?;
    if left.len() != right.len() {
        return Err(EvalError::InvalidMatrix(
            "dot requires vectors with the same number of elements".into(),
        ));
    }
    Ok(left
        .iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum())
}

fn vector_cross(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<f64>, EvalError> {
    let left = vector_values(left)?;
    let right = vector_values(right)?;
    if left.len() != 3 || right.len() != 3 {
        return Err(EvalError::InvalidMatrix(
            "cross requires two vectors with exactly three elements".into(),
        ));
    }
    Ok(vec![
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ])
}

fn vector_norm(matrix: &[Vec<f64>]) -> Result<f64, EvalError> {
    let values = vector_values(matrix)?;
    Ok(values.iter().map(|value| value * value).sum::<f64>().sqrt())
}

fn matrix_lu(matrix: &[Vec<f64>]) -> Result<String, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "lu requires a square matrix".into(),
        ));
    }
    let size = rows;
    let mut upper = matrix.to_vec();
    let mut lower = vec![vec![0.0; size]; size];
    let mut permutation = (0..size).collect::<Vec<_>>();

    for column in 0..size {
        let pivot = (column..size)
            .max_by(|left, right| {
                upper[*left][column]
                    .abs()
                    .total_cmp(&upper[*right][column].abs())
            })
            .unwrap_or(column);
        if upper[pivot][column].abs() < 1e-12 {
            return Err(EvalError::SingularMatrix);
        }
        if pivot != column {
            upper.swap(pivot, column);
            permutation.swap(pivot, column);
            lower.swap(pivot, column);
        }
        for row in (column + 1)..size {
            let factor = upper[row][column] / upper[column][column];
            lower[row][column] = factor;
            for index in column..size {
                upper[row][index] -= factor * upper[column][index];
            }
        }
    }
    for (index, row) in lower.iter_mut().enumerate() {
        row[index] = 1.0;
    }

    let mut p = vec![vec![0.0; size]; size];
    for (row, column) in permutation.iter().enumerate() {
        p[row][*column] = 1.0;
    }

    Ok(format!(
        "L = {}, U = {}, P = {}",
        Value::Matrix(lower).display(),
        Value::Matrix(upper).display(),
        Value::Matrix(p).display()
    ))
}

fn matrix_qr(matrix: &[Vec<f64>]) -> Result<String, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    let mut q = vec![vec![0.0; columns]; rows];
    let mut r = vec![vec![0.0; columns]; columns];

    for column in 0..columns {
        let mut v = (0..rows).map(|row| matrix[row][column]).collect::<Vec<_>>();
        for previous in 0..column {
            let projection = (0..rows)
                .map(|row| q[row][previous] * v[row])
                .sum::<f64>();
            r[previous][column] = projection;
            for row in 0..rows {
                v[row] -= projection * q[row][previous];
            }
        }
        let norm = v.iter().map(|value| value * value).sum::<f64>().sqrt();
        if norm < 1e-12 {
            return Err(EvalError::InvalidMatrix(
                "qr requires linearly independent columns".into(),
            ));
        }
        r[column][column] = norm;
        for row in 0..rows {
            q[row][column] = v[row] / norm;
        }
    }

    Ok(format!(
        "Q = {}, R = {}",
        Value::Matrix(q).display(),
        Value::Matrix(r).display()
    ))
}

fn matrix_rref(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    let mut work = matrix.to_vec();
    let mut pivot_row = 0;

    for column in 0..columns {
        if pivot_row >= rows {
            break;
        }
        let mut selected = None;
        for row in pivot_row..rows {
            if work[row][column].abs() > 1e-12 {
                selected = Some(row);
                break;
            }
        }
        let Some(row) = selected else {
            continue;
        };
        work.swap(pivot_row, row);
        let pivot = work[pivot_row][column];
        for value in &mut work[pivot_row] {
            *value /= pivot;
        }
        for row in 0..rows {
            if row == pivot_row {
                continue;
            }
            let factor = work[row][column];
            if factor.abs() < 1e-15 {
                continue;
            }
            for index in 0..columns {
                work[row][index] -= factor * work[pivot_row][index];
            }
        }
        pivot_row += 1;
    }

    Ok(work)
}

fn matrix_rank(matrix: &[Vec<f64>]) -> Result<usize, EvalError> {
    let reduced = matrix_rref(matrix)?;
    Ok(reduced
        .iter()
        .filter(|row| row.iter().any(|value| value.abs() > 1e-9))
        .count())
}

fn matrix_eigenvalues(matrix: &[Vec<f64>]) -> Result<Vec<f64>, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "eigenvalues requires a square matrix".into(),
        ));
    }
    let size = rows;
    for row in 0..size {
        for column in 0..size {
            if (matrix[row][column] - matrix[column][row]).abs() > 1e-8 {
                return Err(EvalError::InvalidMatrix(
                    "eigenvalues currently supports symmetric matrices".into(),
                ));
            }
        }
    }

    let mut work = matrix.to_vec();
    for _ in 0..(100 * size * size).max(100) {
        let mut largest = 0.0;
        let mut pivot = (0, 1);
        for row in 0..size {
            for column in (row + 1)..size {
                let value = work[row][column].abs();
                if value > largest {
                    largest = value;
                    pivot = (row, column);
                }
            }
        }
        if largest < 1e-10 || size < 2 {
            break;
        }
        let (p, q) = pivot;
        let theta = (work[q][q] - work[p][p]) / (2.0 * work[p][q]);
        let sign = if theta >= 0.0 { 1.0 } else { -1.0 };
        let t = sign / (theta.abs() + (theta * theta + 1.0).sqrt());
        let cosine = 1.0 / (t * t + 1.0).sqrt();
        let sine = t * cosine;

        for index in 0..size {
            if index != p && index != q {
                let value_p = work[index][p];
                let value_q = work[index][q];
                work[index][p] = cosine * value_p - sine * value_q;
                work[p][index] = work[index][p];
                work[index][q] = sine * value_p + cosine * value_q;
                work[q][index] = work[index][q];
            }
        }
        let value_pp = work[p][p];
        let value_qq = work[q][q];
        let value_pq = work[p][q];
        work[p][p] = cosine * cosine * value_pp
            - 2.0 * sine * cosine * value_pq
            + sine * sine * value_qq;
        work[q][q] = sine * sine * value_pp
            + 2.0 * sine * cosine * value_pq
            + cosine * cosine * value_qq;
        work[p][q] = 0.0;
        work[q][p] = 0.0;
    }

    let mut values = (0..size).map(|index| work[index][index]).collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    Ok(values)
}

