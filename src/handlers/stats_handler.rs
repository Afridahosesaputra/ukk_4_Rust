use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Local, NaiveDate};
use crate::auth::get_user_from_token;
use crate::db::DbPool;
use crate::models::{AdminDashboardStats, ApiResponse, SiswaDashboardStats};

const FINE_PER_DAY: f64 = 1000.0;

pub async fn get_admin_stats(
    State(db): State<DbPool>,
    headers: HeaderMap,
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
                        Json(ApiResponse::<AdminDashboardStats> {
                            success: false,
                            message: "Database error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let total_books: i64 = conn.query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0)).unwrap_or(0);
            let total_stock: i64 = conn.query_row("SELECT COALESCE(SUM(total_stock), 0) FROM books", [], |r| r.get(0)).unwrap_or(0);
            let total_members: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE role = 'siswa'", [], |r| r.get(0)).unwrap_or(0);
            let active_borrows: i64 = conn.query_row("SELECT COUNT(*) FROM transactions WHERE status = 'borrowed'", [], |r| r.get(0)).unwrap_or(0);
            let total_returned: i64 = conn.query_row("SELECT COUNT(*) FROM transactions WHERE status = 'returned'", [], |r| r.get(0)).unwrap_or(0);
            
            // Hitung denda dan transaksi overdue
            let today = Local::now().date_naive();
            let mut total_overdue = 0;
            let mut total_fines: f64 = conn.query_row("SELECT COALESCE(SUM(fine_amount), 0.0) FROM transactions WHERE status = 'returned'", [], |r| r.get(0)).unwrap_or(0.0);

            let mut stmt = conn.prepare("SELECT due_date, fine_amount, status FROM transactions WHERE status IN ('borrowed', 'overdue')").unwrap();
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?, row.get::<_, String>(2)?))
            }).unwrap();

            for r in rows {
                if let Ok((due_date, fine, _status)) = r {
                    if let Ok(parsed_due) = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d") {
                        if today > parsed_due {
                            total_overdue += 1;
                            let late_days = (today - parsed_due).num_days();
                            total_fines += (late_days as f64) * FINE_PER_DAY;
                        } else {
                            total_fines += fine;
                        }
                    }
                }
            }

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "Statistik admin berhasil diambil".to_string(),
                    data: Some(AdminDashboardStats {
                        total_books,
                        total_stock,
                        total_members,
                        active_borrows,
                        total_returned,
                        total_overdue,
                        total_fines,
                    }),
                }),
            )
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

pub async fn get_siswa_stats(
    State(db): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(user) => {
            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<SiswaDashboardStats> {
                            success: false,
                            message: "Database error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let active_borrows: i64 = conn.query_row(
                "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND status = 'borrowed'",
                [user.id],
                |r| r.get(0),
            ).unwrap_or(0);

            let total_returned: i64 = conn.query_row(
                "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND status = 'returned'",
                [user.id],
                |r| r.get(0),
            ).unwrap_or(0);

            let today = Local::now().date_naive();
            let mut overdue_count = 0;
            let mut total_fines: f64 = conn.query_row(
                "SELECT COALESCE(SUM(fine_amount), 0.0) FROM transactions WHERE user_id = ?1 AND status = 'returned'",
                [user.id],
                |r| r.get(0),
            ).unwrap_or(0.0);

            let mut stmt = conn.prepare("SELECT due_date, fine_amount FROM transactions WHERE user_id = ?1 AND status IN ('borrowed', 'overdue')").unwrap();
            let rows = stmt.query_map([user.id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            }).unwrap();

            for r in rows {
                if let Ok((due_date, fine)) = r {
                    if let Ok(parsed_due) = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d") {
                        if today > parsed_due {
                            overdue_count += 1;
                            let late_days = (today - parsed_due).num_days();
                            total_fines += (late_days as f64) * FINE_PER_DAY;
                        } else {
                            total_fines += fine;
                        }
                    }
                }
            }

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "Statistik siswa berhasil diambil".to_string(),
                    data: Some(SiswaDashboardStats {
                        active_borrows,
                        total_returned,
                        overdue_count,
                        total_fines,
                    }),
                }),
            )
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                success: false,
                message: "Silakan login".to_string(),
                data: None,
            }),
        ),
    }
}
