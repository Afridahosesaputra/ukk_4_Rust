use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use crate::auth::{get_user_from_token, hash_password};
use crate::db::DbPool;
use crate::models::{ApiResponse, RegisterRequest, UpdateMemberRequest, UserDto};

#[derive(Debug, Deserialize)]
pub struct MemberQuery {
    pub q: Option<String>,
    pub role: Option<String>,
}

pub async fn list_members(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Query(query): Query<MemberQuery>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(user) if user.role == "admin" => {
            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<Vec<UserDto>> {
                            success: false,
                            message: "Database error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let mut sql = String::from(
                "SELECT id, nis_nip, username, full_name, email, phone, class_name, role FROM users WHERE 1=1",
            );
            let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

            if let Some(ref q) = query.q {
                if !q.trim().is_empty() {
                    let search = format!("%{}%", q.trim());
                    sql.push_str(" AND (full_name LIKE ? OR nis_nip LIKE ? OR username LIKE ?)");
                    params_vec.push(search.clone().into());
                    params_vec.push(search.clone().into());
                    params_vec.push(search.into());
                }
            }

            if let Some(ref r) = query.role {
                if !r.trim().is_empty() {
                    sql.push_str(" AND role = ?");
                    params_vec.push(r.trim().to_string().into());
                }
            }

            sql.push_str(" ORDER BY id DESC");

            let mut stmt = match conn.prepare(&sql) {
                Ok(s) => s,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<Vec<UserDto>> {
                            success: false,
                            message: format!("SQL error: {}", e),
                            data: None,
                        }),
                    )
                }
            };

            let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

            let member_iter = match stmt.query_map(params_slice.as_slice(), |row| {
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
            }) {
                Ok(iter) => iter,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<Vec<UserDto>> {
                            success: false,
                            message: format!("Query execution error: {}", e),
                            data: None,
                        }),
                    )
                }
            };

            let mut members = Vec::new();
            for m in member_iter {
                if let Ok(mem) = m {
                    members.push(mem);
                }
            }

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "Data anggota berhasil diambil".to_string(),
                    data: Some(members),
                }),
            )
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak. Fitur ini khusus Admin.".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn create_member(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(user) if user.role == "admin" => {
            let role = payload.role.unwrap_or_else(|| "siswa".to_string());
            let password_hash = match hash_password(&payload.password) {
                Ok(h) => h,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<UserDto> {
                            success: false,
                            message: format!("Gagal enkripsi password: {}", e),
                            data: None,
                        }),
                    )
                }
            };

            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<UserDto> {
                            success: false,
                            message: "Database error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let res = conn.execute(
                "INSERT INTO users (nis_nip, username, password_hash, full_name, email, phone, class_name, role)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    &payload.nis_nip,
                    &payload.username,
                    &password_hash,
                    &payload.full_name,
                    &payload.email,
                    &payload.phone,
                    &payload.class_name,
                    &role,
                ),
            );

            match res {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    (
                        StatusCode::CREATED,
                        Json(ApiResponse {
                            success: true,
                            message: "Anggota baru berhasil ditambahkan!".to_string(),
                            data: Some(UserDto {
                                id,
                                nis_nip: payload.nis_nip,
                                username: payload.username,
                                full_name: payload.full_name,
                                email: payload.email,
                                phone: payload.phone,
                                class_name: payload.class_name,
                                role,
                            }),
                        }),
                    )
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal menambahkan anggota (NIS/Username mungkin sudah ada): {}", e),
                        data: None,
                    }),
                ),
            }
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn update_member(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMemberRequest>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(user) if user.role == "admin" || user.id == id => {
            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()> {
                            success: false,
                            message: "Database lock error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let res = if let Some(ref pass) = payload.password {
                if !pass.trim().is_empty() {
                    let pwd_hash = hash_password(pass.trim()).unwrap_or_default();
                    conn.execute(
                        "UPDATE users SET nis_nip = ?1, full_name = ?2, email = ?3, phone = ?4, class_name = ?5, password_hash = ?6 WHERE id = ?7",
                        (
                            &payload.nis_nip,
                            &payload.full_name,
                            &payload.email,
                            &payload.phone,
                            &payload.class_name,
                            &pwd_hash,
                            id,
                        ),
                    )
                } else {
                    conn.execute(
                        "UPDATE users SET nis_nip = ?1, full_name = ?2, email = ?3, phone = ?4, class_name = ?5 WHERE id = ?6",
                        (
                            &payload.nis_nip,
                            &payload.full_name,
                            &payload.email,
                            &payload.phone,
                            &payload.class_name,
                            id,
                        ),
                    )
                }
            } else {
                conn.execute(
                    "UPDATE users SET nis_nip = ?1, full_name = ?2, email = ?3, phone = ?4, class_name = ?5 WHERE id = ?6",
                    (
                        &payload.nis_nip,
                        &payload.full_name,
                        &payload.email,
                        &payload.phone,
                        &payload.class_name,
                        id,
                    ),
                )
            };

            match res {
                Ok(affected) if affected > 0 => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Data anggota berhasil diperbarui!".to_string(),
                        data: None,
                    }),
                ),
                Ok(_) => (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Anggota tidak ditemukan".to_string(),
                        data: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal update data anggota: {}", e),
                        data: None,
                    }),
                ),
            }
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn delete_member(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(user) if user.role == "admin" => {
            if user.id == id {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: "Tidak dapat menghapus akun admin yang sedang aktif!".to_string(),
                        data: None,
                    }),
                );
            }

            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()> {
                            success: false,
                            message: "Database lock error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            // Cek apakah siswa masih meminjam buku
            let active_borrows: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND status IN ('borrowed', 'overdue')",
                    [id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if active_borrows > 0 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Anggota tidak dapat dihapus karena masih meminjam {} buku.", active_borrows),
                        data: None,
                    }),
                );
            }

            let res = conn.execute("DELETE FROM users WHERE id = ?1", [id]);
            match res {
                Ok(affected) if affected > 0 => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Anggota berhasil dihapus!".to_string(),
                        data: None,
                    }),
                ),
                Ok(_) => (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Anggota tidak ditemukan".to_string(),
                        data: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal menghapus anggota: {}", e),
                        data: None,
                    }),
                ),
            }
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak".to_string(),
                data: None,
            }),
        ),
    }
}
