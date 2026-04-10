pub mod users;

/// Является ли ошибка вставкой уже существующей строки?
pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => {
            db_err.code().as_deref() == Some("23505")
        }
        _ => false,
    }
}
