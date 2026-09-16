use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use crate::auth::{create_session, delete_session, get_user_from_token, hash_password, verify_password};
use crate::db::DbPool;
use crate::models::{ApiResponse, AuthResponse, LoginRequest, RegisterRequest, UserDto};

pub async fn login(
    State(db): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    success: false,
                    message: "Database lock error".to_string(),
                    token: None,
                    user: None,
                }),
            )
        }
    };

    let user_row = conn.query_row(
        "SELECT id, nis_nip, username, password_hash, full_name, email, phone, class_name, role FROM users WHERE username = ?1 OR nis_nip = ?1",
        [&payload.username],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
            ))
        },
    );

    match user_row {
        Ok((id, nis_nip, username, password_hash, full_name, email, phone, class_name, role)) => {
            if verify_password(&payload.password, &password_hash) {
                drop(conn); // release lock before creating session
                match create_session(&db, id) {
                    Ok(token) => {
                        let user_dto = UserDto {
                            id,
                            nis_nip,
                            username,
                            full_name,
                            email,
                            phone,
                            class_name,
                            role,
                        };
                        (
                            StatusCode::OK,
                            Json(AuthResponse {
                                success: true,
                                message: "Login berhasil!".to_string(),
                                token: Some(token),
                                user: Some(user_dto),
                            }),
                        )
                    }
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(AuthResponse {
                            success: false,
                            message: format!("Gagal membuat sesi: {}", e),
                            token: None,
                            user: None,
                        }),
                    ),
                }
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthResponse {
                        success: false,
                        message: "Username/NIS atau Password salah!".to_string(),
                        token: None,
                        user: None,
                    }),
                )
            }
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                success: false,
                message: "Akun tidak ditemukan. Silakan periksa kembali atau daftar akun baru.".to_string(),
                token: None,
                user: None,
            }),
        ),
    }
}

pub async fn register(
    State(db): State<DbPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let role = payload.role.unwrap_or_else(|| "siswa".to_string());
    
    // Hash password
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<UserDto> {
                    success: false,
                    message: format!("Gagal mengenkripsi kata sandi: {}", e),
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
                    message: "Database lock error".to_string(),
                    data: None,
                }),
            )
        }
    };

    let insert_res = conn.execute(
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

    match insert_res {
        Ok(_) => {
            let last_id = conn.last_insert_rowid();
            let user_dto = UserDto {
                id: last_id,
                nis_nip: payload.nis_nip,
                username: payload.username,
                full_name: payload.full_name,
                email: payload.email,
                phone: payload.phone,
                class_name: payload.class_name,
                role,
            };
            (
                StatusCode::CREATED,
                Json(ApiResponse {
                    success: true,
                    message: "Pendaftaran akun berhasil! Silakan login.".to_string(),
                    data: Some(user_dto),
                }),
            )
        }
        Err(e) => {
            let err_msg = e.to_string();
            let message = if err_msg.contains("UNIQUE constraint failed: users.username") {
                "Username sudah digunakan, silakan gunakan username lain.".to_string()
            } else if err_msg.contains("UNIQUE constraint failed: users.nis_nip") {
                "NIS / NIP sudah terdaftar di sistem!".to_string()
            } else {
                format!("Pendaftaran gagal: {}", err_msg)
            };
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message,
                    data: None,
                }),
            )
        }
    }
}

pub async fn get_me(
    State(db): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    if let Some(tok) = token {
        if let Some(user) = get_user_from_token(&db, tok) {
            return (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "Data user berhasil diambil".to_string(),
                    data: Some(user),
                }),
            );
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(ApiResponse {
            success: false,
            message: "Sesi telah berakhir atau tidak valid. Silakan login kembali.".to_string(),
            data: None,
        }),
    )
}

pub async fn logout(
    State(db): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    if let Some(tok) = token {
        let _ = delete_session(&db, tok);
    }

    (
        StatusCode::OK,
        Json(ApiResponse::<()> {
            success: true,
            message: "Logout berhasil.".to_string(),
            data: None,
        }),
    )
}
