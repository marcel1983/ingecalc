fn matrix_shape(matrix: &[Vec<f64>]) -> Result<(usize, usize), EvalError> {
    let rows = matrix.len();
    let columns = matrix.first().map(Vec::len).unwrap_or(0);
    if rows == 0 || columns == 0 || matrix.iter().any(|row| row.len() != columns) {
        return Err(EvalError::InvalidMatrix(
            "expected a non-empty rectangular matrix".into(),
        ));
    }
    Ok((rows, columns))
}

fn matrix_transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let (rows, columns) = matrix_shape(matrix).unwrap_or((0, 0));
    (0..columns)
        .map(|column| (0..rows).map(|row| matrix[row][column]).collect())
        .collect()
}

fn matrix_add(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (left_rows, left_columns) = matrix_shape(left)?;
    let (right_rows, right_columns) = matrix_shape(right)?;
    if left_rows != right_rows || left_columns != right_columns {
        return Err(EvalError::InvalidMatrix(
            "matrix addition requires the same dimensions".into(),
        ));
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(left, right)| {
            left.iter()
                .zip(right)
                .map(|(left, right)| left + right)
                .collect()
        })
        .collect())
}

fn matrix_sub(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (left_rows, left_columns) = matrix_shape(left)?;
    let (right_rows, right_columns) = matrix_shape(right)?;
    if left_rows != right_rows || left_columns != right_columns {
        return Err(EvalError::InvalidMatrix(
            "matrix subtraction requires the same dimensions".into(),
        ));
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(left, right)| {
            left.iter()
                .zip(right)
                .map(|(left, right)| left - right)
                .collect()
        })
        .collect())
}

fn matrix_scalar(matrix: &[Vec<f64>], factor: f64) -> Vec<Vec<f64>> {
    matrix
        .iter()
        .map(|row| row.iter().map(|value| value * factor).collect())
        .collect()
}

fn matrix_mul(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (left_rows, left_columns) = matrix_shape(left)?;
    let (right_rows, right_columns) = matrix_shape(right)?;
    if left_columns != right_rows {
        return Err(EvalError::InvalidMatrix(
            "matrix multiplication requires compatible dimensions".into(),
        ));
    }
    let mut result = vec![vec![0.0; right_columns]; left_rows];
    for row in 0..left_rows {
        for column in 0..right_columns {
            result[row][column] = (0..left_columns)
                .map(|index| left[row][index] * right[index][column])
                .sum();
        }
    }
    Ok(result)
}

fn matrix_power(matrix: &[Vec<f64>], exponent: i32) -> Result<Vec<Vec<f64>>, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "matrix powers require a square matrix".into(),
        ));
    }
    if exponent == 0 {
        return Ok(identity_matrix(rows));
    }
    let base = if exponent < 0 {
        matrix_inverse(matrix)?
    } else {
        matrix.to_vec()
    };
    let mut result = identity_matrix(rows);
    let mut power = exponent.unsigned_abs();
    let mut factor = base;
    while power > 0 {
        if power % 2 == 1 {
            result = matrix_mul(&result, &factor)?;
        }
        power /= 2;
        if power > 0 {
            factor = matrix_mul(&factor, &factor)?;
        }
    }
    Ok(result)
}

fn identity_matrix(size: usize) -> Vec<Vec<f64>> {
    (0..size)
        .map(|row| {
            (0..size)
                .map(|column| if row == column { 1.0 } else { 0.0 })
                .collect()
        })
        .collect()
}

fn matrix_det(matrix: &[Vec<f64>]) -> Result<f64, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "determinant requires a square matrix".into(),
        ));
    }
    let mut work = matrix.to_vec();
    let mut determinant = 1.0;
    for column in 0..rows {
        let pivot = (column..rows)
            .max_by(|left, right| {
                work[*left][column]
                    .abs()
                    .total_cmp(&work[*right][column].abs())
            })
            .unwrap_or(column);
        if work[pivot][column].abs() < 1e-12 {
            return Ok(0.0);
        }
        if pivot != column {
            work.swap(pivot, column);
            determinant = -determinant;
        }
        determinant *= work[column][column];
        let pivot_value = work[column][column];
        for row in (column + 1)..rows {
            let factor = work[row][column] / pivot_value;
            for index in column..rows {
                work[row][index] -= factor * work[column][index];
            }
        }
    }
    Ok(determinant)
}

fn matrix_inverse(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "inverse requires a square matrix".into(),
        ));
    }
    let size = rows;
    let mut work = matrix
        .iter()
        .enumerate()
        .map(|(row_index, values)| {
            let mut row = values.clone();
            row.extend(
                (0..size).map(|column| if row_index == column { 1.0 } else { 0.0 }),
            );
            row
        })
        .collect::<Vec<_>>();

    for column in 0..size {
        let pivot = (column..size)
            .max_by(|left, right| {
                work[*left][column]
                    .abs()
                    .total_cmp(&work[*right][column].abs())
            })
            .unwrap_or(column);
        if work[pivot][column].abs() < 1e-12 {
            return Err(EvalError::SingularMatrix);
        }
        work.swap(pivot, column);
        let pivot_value = work[column][column];
        for value in &mut work[column] {
            *value /= pivot_value;
        }
        for row in 0..size {
            if row == column {
                continue;
            }
            let factor = work[row][column];
            for index in 0..(2 * size) {
                work[row][index] -= factor * work[column][index];
            }
        }
    }

    Ok(work
        .into_iter()
        .map(|row| row[size..].to_vec())
        .collect())
}

fn matrix_solve(matrix: &[Vec<f64>], vector: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "solve requires a square matrix".into(),
        ));
    }
    let (vector_rows, vector_columns) = matrix_shape(vector)?;
    if vector_rows != rows || vector_columns != 1 {
        return Err(EvalError::InvalidMatrix(
            "solve expects a column vector with the same number of rows".into(),
        ));
    }

    let size = rows;
    let mut work = matrix
        .iter()
        .enumerate()
        .map(|(row_index, values)| {
            let mut row = values.clone();
            row.push(vector[row_index][0]);
            row
        })
        .collect::<Vec<_>>();

    for column in 0..size {
        let pivot = (column..size)
            .max_by(|left, right| {
                work[*left][column]
                    .abs()
                    .total_cmp(&work[*right][column].abs())
            })
            .unwrap_or(column);
        if work[pivot][column].abs() < 1e-12 {
            return Err(EvalError::SingularMatrix);
        }
        work.swap(pivot, column);
        let pivot_value = work[column][column];
        for value in &mut work[column] {
            *value /= pivot_value;
        }
        for row in 0..size {
            if row == column {
                continue;
            }
            let factor = work[row][column];
            for index in column..=size {
                work[row][index] -= factor * work[column][index];
            }
        }
    }

    Ok(work.into_iter().map(|row| vec![row[size]]).collect())
}

fn matrix_trace(matrix: &[Vec<f64>]) -> Result<f64, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if rows != columns {
        return Err(EvalError::InvalidMatrix(
            "trace requires a square matrix".into(),
        ));
    }
    Ok((0..rows).map(|index| matrix[index][index]).sum())
}

fn matrix_row(matrix: &[Vec<f64>], row: usize) -> Result<Vec<Vec<f64>>, EvalError> {
    let (rows, _) = matrix_shape(matrix)?;
    if row == 0 || row > rows {
        return Err(EvalError::IndexOutOfBounds(format!(
            "row {row} is outside 1..={rows}"
        )));
    }
    Ok(vec![matrix[row - 1].clone()])
}

fn matrix_col(matrix: &[Vec<f64>], column: usize) -> Result<Vec<Vec<f64>>, EvalError> {
    let (_, columns) = matrix_shape(matrix)?;
    if column == 0 || column > columns {
        return Err(EvalError::IndexOutOfBounds(format!(
            "column {column} is outside 1..={columns}"
        )));
    }
    Ok(matrix.iter().map(|row| vec![row[column - 1]]).collect())
}

fn matrix_augment(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (left_rows, _) = matrix_shape(left)?;
    let (right_rows, _) = matrix_shape(right)?;
    if left_rows != right_rows {
        return Err(EvalError::InvalidMatrix(
            "augment requires matrices with the same number of rows".into(),
        ));
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(left, right)| {
            let mut row = left.clone();
            row.extend_from_slice(right);
            row
        })
        .collect())
}

fn matrix_stack(top: &[Vec<f64>], bottom: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, EvalError> {
    let (_, top_columns) = matrix_shape(top)?;
    let (_, bottom_columns) = matrix_shape(bottom)?;
    if top_columns != bottom_columns {
        return Err(EvalError::InvalidMatrix(
            "stack requires matrices with the same number of columns".into(),
        ));
    }
    let mut rows = top.to_vec();
    rows.extend_from_slice(bottom);
    Ok(rows)
}

fn matrix_minor(
    matrix: &[Vec<f64>],
    row: usize,
    column: usize,
) -> Result<Vec<Vec<f64>>, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    if row == 0 || row > rows || column == 0 || column > columns {
        return Err(EvalError::IndexOutOfBounds(format!(
            "minor index ({row}, {column}) is outside the matrix"
        )));
    }
    if rows == 1 || columns == 1 {
        return Err(EvalError::InvalidMatrix(
            "minor requires a matrix with at least two rows and columns".into(),
        ));
    }
    Ok(matrix
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != row - 1)
        .map(|(_, values)| {
            values
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != column - 1)
                .map(|(_, value)| *value)
                .collect()
        })
        .collect())
}

fn matrix_cofactor(matrix: &[Vec<f64>], row: usize, column: usize) -> Result<f64, EvalError> {
    let minor = matrix_minor(matrix, row, column)?;
    let determinant = matrix_det(&minor)?;
    Ok(if (row + column).is_multiple_of(2) {
        determinant
    } else {
        -determinant
    })
}

fn matrix_sum(matrix: &[Vec<f64>]) -> Result<f64, EvalError> {
    matrix_shape(matrix)?;
    Ok(matrix.iter().flatten().copied().sum::<f64>())
}

fn matrix_mean(matrix: &[Vec<f64>]) -> Result<f64, EvalError> {
    let (rows, columns) = matrix_shape(matrix)?;
    Ok(matrix_sum(matrix)? / (rows.saturating_mul(columns)) as f64)
}
