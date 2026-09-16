use crate::db::DbPool;
use crate::models::UserDto;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub fn create_session(db: &DbPool, user_id: i64) -> Result<String, String> {
    let token = Uuid::new_v4().to_string();
    let conn = db.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO user_tokens (token, user_id) VALUES (?1, ?2)",
        (&token, user_id),
    ).map_err(|e| e.to_string())?;

    Ok(token)
}

pub fn get_user_from_token(db: &DbPool, token: &str) -> Option<UserDto> {
    let conn = db.lock().ok()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT u.id, u.nis_nip, u.username, u.full_name, u.email, u.phone, u.class_name, u.role
        FROM user_tokens t
        JOIN users u ON t.user_id = u.id
        WHERE t.token = ?1
        "#
    ).ok()?;

    stmt.query_row([token], |row| {
        Ok(UserDto {
            id: row.get(0)?,
            nis_nip: row.get(1)?,
            username: row.get(2)?,
            full_name: row.get(3)?,
            email: row.get(4)?,
            phone: row.get(5)?,
            class_name: row.get(6)?,
            role: row.get(7)?,
        })
    }).ok()
}

pub fn delete_session(db: &DbPool, token: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM user_tokens WHERE token = ?1", [token])
        .map_err(|e| e.to_string())?;
    Ok(())
}
