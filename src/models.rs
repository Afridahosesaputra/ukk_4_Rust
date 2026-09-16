use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub nis_nip: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub class_name: Option<String>,
    pub role: String, // "admin" | "siswa"
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Book {
    pub id: i64,
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub year: i32,
    pub category_id: i64,
    pub category_name: Option<String>,
    pub stock: i32,
    pub total_stock: i32,
    pub shelf_location: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: i64,
    pub transaction_code: String,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub nis_nip: Option<String>,
    pub book_id: i64,
    pub book_title: Option<String>,
    pub book_isbn: Option<String>,
    pub borrow_date: String,
    pub due_date: String,
    pub return_date: Option<String>,
    pub status: String, // "borrowed", "returned", "overdue"
    pub fine_amount: f64,
    pub notes: Option<String>,
    pub created_at: String,
}

// Request & Response DTOs
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user: Option<UserDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: i64,
    pub nis_nip: String,
    pub username: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub class_name: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub nis_nip: String,
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub class_name: Option<String>,
    pub role: Option<String>, // Default "siswa"
}

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub year: i32,
    pub category_id: i64,
    pub stock: i32,
    pub shelf_location: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub year: i32,
    pub category_id: i64,
    pub stock: i32,
    pub total_stock: i32,
    pub shelf_location: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BorrowBookRequest {
    pub book_id: i64,
    pub user_id: Option<i64>, // If admin borrows on behalf of a student
    pub borrow_duration_days: Option<i64>, // Default 7 days
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReturnBookRequest {
    pub transaction_id: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub nis_nip: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub class_name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct AdminDashboardStats {
    pub total_books: i64,
    pub total_stock: i64,
    pub total_members: i64,
    pub active_borrows: i64,
    pub total_returned: i64,
    pub total_overdue: i64,
    pub total_fines: f64,
}

#[derive(Debug, Serialize)]
pub struct SiswaDashboardStats {
    pub active_borrows: i64,
    pub total_returned: i64,
    pub overdue_count: i64,
    pub total_fines: f64,
}
