use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Local, NaiveDate};
use serde::Deserialize;
use uuid::Uuid;
use crate::auth::get_user_from_token;
use crate::db::DbPool;
use crate::models::{ApiResponse, BorrowBookRequest, ReturnBookRequest, Transaction};

const FINE_PER_DAY: f64 = 1000.0; // Denda Rp 1.000 / hari keterlambatan

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub status: Option<String>,
    pub q: Option<String>,
}

pub async fn list_transactions(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Query(query): Query<TransactionQuery>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    let current_user = match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(u) => u,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<Vec<Transaction>> {
                    success: false,
                    message: "Silakan login terlebih dahulu".to_string(),
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
                Json(ApiResponse::<Vec<Transaction>> {
                    success: false,
                    message: "Database error".to_string(),
                    data: None,
                }),
            )
        }
    };

    let mut sql = String::from(
        r#"
        SELECT t.id, t.transaction_code, t.user_id, u.full_name as user_name, u.nis_nip,
               t.book_id, b.title as book_title, b.isbn as book_isbn,
               t.borrow_date, t.due_date, t.return_date, t.status, t.fine_amount, t.notes, t.created_at
        FROM transactions t
        JOIN users u ON t.user_id = u.id
        JOIN books b ON t.book_id = b.id
        WHERE 1=1
        "#,
    );

    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    // Jika siswa biasa, hanya tampilkan transaksinya sendiri
    if current_user.role == "siswa" {
        sql.push_str(" AND t.user_id = ?");
        params_vec.push(current_user.id.into());
    }

    if let Some(ref st) = query.status {
        if !st.trim().is_empty() && st.trim() != "all" {
            sql.push_str(" AND t.status = ?");
            params_vec.push(st.trim().to_string().into());
        }
    }

    if let Some(ref q) = query.q {
        if !q.trim().is_empty() {
            let search = format!("%{}%", q.trim());
            sql.push_str(" AND (t.transaction_code LIKE ? OR u.full_name LIKE ? OR u.nis_nip LIKE ? OR b.title LIKE ? OR b.isbn LIKE ?)");
            params_vec.push(search.clone().into());
            params_vec.push(search.clone().into());
            params_vec.push(search.clone().into());
            params_vec.push(search.clone().into());
            params_vec.push(search.into());
        }
    }

    sql.push_str(" ORDER BY t.id DESC");

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Transaction>> {
                    success: false,
                    message: format!("SQL Error: {}", e),
                    data: None,
                }),
            )
        }
    };

    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

    let today = Local::now().date_naive();

    let trx_iter = match stmt.query_map(params_slice.as_slice(), |row| {
        let id: i64 = row.get(0)?;
        let code: String = row.get(1)?;
        let user_id: i64 = row.get(2)?;
        let user_name: Option<String> = row.get(3)?;
        let nis_nip: Option<String> = row.get(4)?;
        let book_id: i64 = row.get(5)?;
        let book_title: Option<String> = row.get(6)?;
        let book_isbn: Option<String> = row.get(7)?;
        let borrow_date: String = row.get(8)?;
        let due_date: String = row.get(9)?;
        let return_date: Option<String> = row.get(10)?;
        let mut status: String = row.get(11)?;
        let mut fine_amount: f64 = row.get(12)?;
        let notes: Option<String> = row.get(13)?;
        let created_at: String = row.get(14)?;

        // Auto kalkulasi denda real-time jika masih 'borrowed' tapi sudah melewati due_date
        if status == "borrowed" {
            if let Ok(parsed_due) = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d") {
                if today > parsed_due {
                    let late_days = (today - parsed_due).num_days();
                    if late_days > 0 {
                        status = "overdue".to_string();
                        fine_amount = (late_days as f64) * FINE_PER_DAY;
                    }
                }
            }
        }

        Ok(Transaction {
            id,
            transaction_code: code,
            user_id,
            user_name,
            nis_nip,
            book_id,
            book_title,
            book_isbn,
            borrow_date,
            due_date,
            return_date,
            status,
            fine_amount,
            notes,
            created_at,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Transaction>> {
                    success: false,
                    message: format!("Query Error: {}", e),
                    data: None,
                }),
            )
        }
    };

    let mut list = Vec::new();
    for t in trx_iter {
        if let Ok(trx) = t {
            list.push(trx);
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Data transaksi berhasil diambil".to_string(),
            data: Some(list),
        }),
    )
}

pub async fn borrow_book(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<BorrowBookRequest>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    let current_user = match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(u) => u,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<Transaction> {
                    success: false,
                    message: "Silakan login terlebih dahulu".to_string(),
                    data: None,
                }),
            )
        }
    };

    // Target user: Siswa yang sedang login, atau target user_id yang dipilih admin
    let target_user_id = if current_user.role == "admin" {
        payload.user_id.unwrap_or(current_user.id)
    } else {
        current_user.id
    };

    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Transaction> {
                    success: false,
                    message: "Database error".to_string(),
                    data: None,
                }),
            )
        }
    };

    // 1. Cek stok buku
    let book_info: rusqlite::Result<(String, String, i32)> = conn.query_row(
        "SELECT title, isbn, stock FROM books WHERE id = ?1",
        [payload.book_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    );

    let (book_title, book_isbn, stock) = match book_info {
        Ok(info) => info,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Buku tidak ditemukan!".to_string(),
                    data: None,
                }),
            )
        }
    };

    if stock <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!("Buku '{}' sedang habis (Stok: 0)!", book_title),
                data: None,
            }),
        );
    }

    // 2. Cek apakah user sedang meminjam buku yang sama dan belum dikembalikan
    let active_same_book: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND book_id = ?2 AND status IN ('borrowed', 'overdue')",
            [target_user_id, payload.book_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if active_same_book > 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Anda masih meminjam buku ini. Harap kembalikan terlebih dahulu sebelum meminjam ulang.".to_string(),
                data: None,
            }),
        );
    }

    // 3. Batas maksimal peminjaman aktif per siswa (misal: 3 buku)
    let total_active_borrows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM transactions WHERE user_id = ?1 AND status IN ('borrowed', 'overdue')",
            [target_user_id,],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if total_active_borrows >= 3 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Maksimal peminjaman aktif adalah 3 buku. Harap kembalikan buku sebelumnya terlebih dahulu.".to_string(),
                data: None,
            }),
        );
    }

    // Generate Transaction Code
    let duration_days = payload.borrow_duration_days.unwrap_or(7);
    let today = Local::now().date_naive();
    let due_date = today + chrono::Duration::days(duration_days);

    let random_suffix = &Uuid::new_v4().to_string()[..4].to_uppercase();
    let trx_code = format!(
        "TRX-{}{:02}{:02}-{}",
        today.year(),
        today.month(),
        today.day(),
        random_suffix
    );

    let borrow_date_str = today.format("%Y-%m-%d").to_string();
    let due_date_str = due_date.format("%Y-%m-%d").to_string();

    // Insert transaction & decrement book stock in an atomic block
    let insert_res = conn.execute(
        r#"
        INSERT INTO transactions (transaction_code, user_id, book_id, borrow_date, due_date, status, fine_amount, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, 'borrowed', 0.0, ?6)
        "#,
        (
            &trx_code,
            target_user_id,
            payload.book_id,
            &borrow_date_str,
            &due_date_str,
            &payload.notes,
        ),
    );

    match insert_res {
        Ok(_) => {
            let trx_id = conn.last_insert_rowid();
            // Kurangi stok buku
            let _ = conn.execute("UPDATE books SET stock = stock - 1 WHERE id = ?1", [payload.book_id]);

            // Ambil target user detail
            let (target_name, target_nis): (String, String) = conn
                .query_row("SELECT full_name, nis_nip FROM users WHERE id = ?1", [target_user_id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .unwrap_or_default();

            (
                StatusCode::CREATED,
                Json(ApiResponse {
                    success: true,
                    message: format!("Buku '{}' berhasil dipinjam! Batas pengembalian: {}", book_title, due_date_str),
                    data: Some(Transaction {
                        id: trx_id,
                        transaction_code: trx_code,
                        user_id: target_user_id,
                        user_name: Some(target_name),
                        nis_nip: Some(target_nis),
                        book_id: payload.book_id,
                        book_title: Some(book_title),
                        book_isbn: Some(book_isbn),
                        borrow_date: borrow_date_str,
                        due_date: due_date_str,
                        return_date: None,
                        status: "borrowed".to_string(),
                        fine_amount: 0.0,
                        notes: payload.notes,
                        created_at: Local::now().to_rfc3339(),
                    }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Gagal memproses peminjaman: {}", e),
                data: None,
            }),
        ),
    }
}

pub async fn return_book(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<ReturnBookRequest>,
) -> impl IntoResponse {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").trim());

    let current_user = match token.and_then(|t| get_user_from_token(&db, t)) {
        Some(u) => u,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<Transaction> {
                    success: false,
                    message: "Silakan login terlebih dahulu".to_string(),
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
                Json(ApiResponse::<Transaction> {
                    success: false,
                    message: "Database error".to_string(),
                    data: None,
                }),
            )
        }
    };

    // Ambil data transaksi
    let trx_row: rusqlite::Result<(i64, i64, String, String, String)> = conn.query_row(
        "SELECT id, user_id, book_id, due_date, status FROM transactions WHERE id = ?1",
        [payload.transaction_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    );

    let (trx_id, user_id, book_id, due_date, status) = match trx_row {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Data peminjaman tidak ditemukan".to_string(),
                    data: None,
                }),
            )
        }
    };

    // Validasi otorisasi: hanya peminjam atau admin yang bisa mengembalikan
    if current_user.role != "admin" && current_user.id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Anda tidak memiliki izin mengembalikan peminjaman ini.".to_string(),
                data: None,
            }),
        );
    }

    if status == "returned" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: "Buku ini sudah dikembalikan sebelumnya!".to_string(),
                data: None,
            }),
        );
    }

    // Hitung denda jika ada keterlambatan
    let today = Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    let mut fine_amount = 0.0;
    if let Ok(parsed_due) = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d") {
        if today > parsed_due {
            let late_days = (today - parsed_due).num_days();
            if late_days > 0 {
                fine_amount = (late_days as f64) * FINE_PER_DAY;
            }
        }
    }

    // Update status transaksi
    let update_res = conn.execute(
        r#"
        UPDATE transactions
        SET return_date = ?1, status = 'returned', fine_amount = ?2, notes = COALESCE(?3, notes)
        WHERE id = ?4
        "#,
        (&today_str, fine_amount, &payload.notes, trx_id),
    );

    match update_res {
        Ok(_) => {
            // Tambah stok buku kembali
            let _ = conn.execute("UPDATE books SET stock = stock + 1 WHERE id = ?1", [book_id]);

            let fine_msg = if fine_amount > 0.0 {
                format!(" Dikenakan denda keterlambatan sebesar Rp {:.0}.", fine_amount)
            } else {
                " Tepat waktu tanpa denda.".to_string()
            };

            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: format!("Buku berhasil dikembalikan!{}", fine_msg),
                    data: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Gagal mengembalikan buku: {}", e),
                data: None,
            }),
        ),
    }
}

pub async fn delete_transaction(
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

            // Jika status masih 'borrowed' / 'overdue', kembalikan stok buku sebelum hapus
            let trx_info: rusqlite::Result<(i64, String)> = conn.query_row(
                "SELECT book_id, status FROM transactions WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            );

            if let Ok((book_id, status)) = trx_info {
                if status == "borrowed" || status == "overdue" {
                    let _ = conn.execute("UPDATE books SET stock = stock + 1 WHERE id = ?1", [book_id]);
                }
            }

            let res = conn.execute("DELETE FROM transactions WHERE id = ?1", [id]);
            match res {
                Ok(affected) if affected > 0 => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Data riwayat transaksi berhasil dihapus!".to_string(),
                        data: None,
                    }),
                ),
                Ok(_) => (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Transaksi tidak ditemukan".to_string(),
                        data: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal menghapus transaksi: {}", e),
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
