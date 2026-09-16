use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use crate::auth::get_user_from_token;
use crate::db::DbPool;
use crate::models::{ApiResponse, Book, Category, CreateBookRequest, CreateCategoryRequest, UpdateBookRequest};

#[derive(Debug, Deserialize)]
pub struct BookQuery {
    pub q: Option<String>,
    pub category_id: Option<i64>,
}

pub async fn list_books(
    State(db): State<DbPool>,
    Query(query): Query<BookQuery>,
) -> impl IntoResponse {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Book>> {
                    success: false,
                    message: "Database error".to_string(),
                    data: None,
                }),
            )
        }
    };

    let mut sql = String::from(
        r#"
        SELECT b.id, b.isbn, b.title, b.author, b.publisher, b.year, b.category_id,
               c.name as category_name, b.stock, b.total_stock, b.shelf_location,
               b.description, b.cover_url, b.created_at
        FROM books b
        LEFT JOIN categories c ON b.category_id = c.id
        WHERE 1=1
        "#,
    );

    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(ref q) = query.q {
        if !q.trim().is_empty() {
            let search_pat = format!("%{}%", q.trim());
            sql.push_str(" AND (b.title LIKE ? OR b.author LIKE ? OR b.isbn LIKE ? OR b.publisher LIKE ?)");
            params_vec.push(search_pat.clone().into());
            params_vec.push(search_pat.clone().into());
            params_vec.push(search_pat.clone().into());
            params_vec.push(search_pat.into());
        }
    }

    if let Some(cat_id) = query.category_id {
        if cat_id > 0 {
            sql.push_str(" AND b.category_id = ?");
            params_vec.push(cat_id.into());
        }
    }

    sql.push_str(" ORDER BY b.id DESC");

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Book>> {
                    success: false,
                    message: format!("SQL Error: {}", e),
                    data: None,
                }),
            )
        }
    };

    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

    let book_iter = match stmt.query_map(params_slice.as_slice(), |row| {
        Ok(Book {
            id: row.get(0)?,
            isbn: row.get(1)?,
            title: row.get(2)?,
            author: row.get(3)?,
            publisher: row.get(4)?,
            year: row.get(5)?,
            category_id: row.get(6)?,
            category_name: row.get(7)?,
            stock: row.get(8)?,
            total_stock: row.get(9)?,
            shelf_location: row.get(10)?,
            description: row.get(11)?,
            cover_url: row.get(12)?,
            created_at: row.get(13)?,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Book>> {
                    success: false,
                    message: format!("Query execution error: {}", e),
                    data: None,
                }),
            )
        }
    };

    let mut books = Vec::new();
    for book in book_iter {
        if let Ok(b) = book {
            books.push(b);
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Berhasil mengambil data buku".to_string(),
            data: Some(books),
        }),
    )
}

pub async fn get_book_detail(
    State(db): State<DbPool>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Book> {
                    success: false,
                    message: "Database error".to_string(),
                    data: None,
                }),
            )
        }
    };

    let book_res = conn.query_row(
        r#"
        SELECT b.id, b.isbn, b.title, b.author, b.publisher, b.year, b.category_id,
               c.name as category_name, b.stock, b.total_stock, b.shelf_location,
               b.description, b.cover_url, b.created_at
        FROM books b
        LEFT JOIN categories c ON b.category_id = c.id
        WHERE b.id = ?1
        "#,
        [id],
        |row| {
            Ok(Book {
                id: row.get(0)?,
                isbn: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                publisher: row.get(4)?,
                year: row.get(5)?,
                category_id: row.get(6)?,
                category_name: row.get(7)?,
                stock: row.get(8)?,
                total_stock: row.get(9)?,
                shelf_location: row.get(10)?,
                description: row.get(11)?,
                cover_url: row.get(12)?,
                created_at: row.get(13)?,
            })
        },
    );

    match book_res {
        Ok(book) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Detail buku ditemukan".to_string(),
                data: Some(book),
            }),
        ),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                message: "Buku tidak ditemukan".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn create_book(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<CreateBookRequest>,
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
                        Json(ApiResponse::<Book> {
                            success: false,
                            message: "Database lock error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let res = conn.execute(
                r#"
                INSERT INTO books (isbn, title, author, publisher, year, category_id, stock, total_stock, shelf_location, description, cover_url)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                (
                    &payload.isbn,
                    &payload.title,
                    &payload.author,
                    &payload.publisher,
                    payload.year,
                    payload.category_id,
                    payload.stock,
                    payload.stock, // total_stock initially equals stock
                    &payload.shelf_location,
                    &payload.description,
                    &payload.cover_url,
                ),
            );

            match res {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    (
                        StatusCode::CREATED,
                        Json(ApiResponse {
                            success: true,
                            message: "Data buku berhasil ditambahkan!".to_string(),
                            data: Some(Book {
                                id,
                                isbn: payload.isbn,
                                title: payload.title,
                                author: payload.author,
                                publisher: payload.publisher,
                                year: payload.year,
                                category_id: payload.category_id,
                                category_name: None,
                                stock: payload.stock,
                                total_stock: payload.stock,
                                shelf_location: payload.shelf_location,
                                description: payload.description,
                                cover_url: payload.cover_url,
                                created_at: chrono::Local::now().to_rfc3339(),
                            }),
                        }),
                    )
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal menambahkan buku: ISBN mungkin sudah terdaftar ({})", e),
                        data: None,
                    }),
                ),
            }
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak. Fitur ini hanya untuk Admin Perpustakaan.".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn update_book(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateBookRequest>,
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

            let res = conn.execute(
                r#"
                UPDATE books
                SET isbn = ?1, title = ?2, author = ?3, publisher = ?4, year = ?5,
                    category_id = ?6, stock = ?7, total_stock = ?8, shelf_location = ?9,
                    description = ?10, cover_url = ?11
                WHERE id = ?12
                "#,
                (
                    &payload.isbn,
                    &payload.title,
                    &payload.author,
                    &payload.publisher,
                    payload.year,
                    payload.category_id,
                    payload.stock,
                    payload.total_stock,
                    &payload.shelf_location,
                    &payload.description,
                    &payload.cover_url,
                    id,
                ),
            );

            match res {
                Ok(affected) if affected > 0 => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Data buku berhasil diperbarui!".to_string(),
                        data: None,
                    }),
                ),
                Ok(_) => (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Buku tidak ditemukan".to_string(),
                        data: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal update buku: {}", e),
                        data: None,
                    }),
                ),
            }
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak. Fitur ini hanya untuk Admin Perpustakaan.".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn delete_book(
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

            // Cek apakah buku sedang dipinjam
            let active_borrows: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM transactions WHERE book_id = ?1 AND status IN ('borrowed', 'overdue')",
                    [id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if active_borrows > 0 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Buku tidak dapat dihapus karena sedang dipinjam ({} transaksi aktif).", active_borrows),
                        data: None,
                    }),
                );
            }

            let res = conn.execute("DELETE FROM books WHERE id = ?1", [id]);

            match res {
                Ok(affected) if affected > 0 => (
                    StatusCode::OK,
                    Json(ApiResponse {
                        success: true,
                        message: "Data buku berhasil dihapus!".to_string(),
                        data: None,
                    }),
                ),
                Ok(_) => (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        success: false,
                        message: "Buku tidak ditemukan".to_string(),
                        data: None,
                    }),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal menghapus buku: {}", e),
                        data: None,
                    }),
                ),
            }
        }
        _ => (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak. Fitur ini hanya untuk Admin Perpustakaan.".to_string(),
                data: None,
            }),
        ),
    }
}

pub async fn list_categories(State(db): State<DbPool>) -> impl IntoResponse {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Category>> {
                    success: false,
                    message: "Database error".to_string(),
                    data: None,
                }),
            )
        }
    };

    let mut stmt = match conn.prepare("SELECT id, name, description FROM categories ORDER BY name ASC") {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Category>> {
                    success: false,
                    message: format!("SQL Error: {}", e),
                    data: None,
                }),
            )
        }
    };

    let cat_iter = match stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<Category>> {
                    success: false,
                    message: format!("Error: {}", e),
                    data: None,
                }),
            )
        }
    };

    let mut categories = Vec::new();
    for c in cat_iter {
        if let Ok(cat) = c {
            categories.push(cat);
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Berhasil mengambil kategori".to_string(),
            data: Some(categories),
        }),
    )
}

pub async fn create_category(
    State(db): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<CreateCategoryRequest>,
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
                        Json(ApiResponse::<Category> {
                            success: false,
                            message: "Database error".to_string(),
                            data: None,
                        }),
                    )
                }
            };

            let res = conn.execute(
                "INSERT INTO categories (name, description) VALUES (?1, ?2)",
                (&payload.name, &payload.description),
            );

            match res {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    (
                        StatusCode::CREATED,
                        Json(ApiResponse {
                            success: true,
                            message: "Kategori berhasil ditambahkan".to_string(),
                            data: Some(Category {
                                id,
                                name: payload.name,
                                description: payload.description,
                            }),
                        }),
                    )
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse {
                        success: false,
                        message: format!("Gagal menambahkan kategori: {}", e),
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
