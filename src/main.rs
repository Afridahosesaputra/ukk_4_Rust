mod auth;
mod db;
mod handlers;
mod models;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use handlers::{
    auth_handler::{get_me, login, logout, register},
    book_handler::{create_book, create_category, delete_book, get_book_detail, list_books, list_categories, update_book},
    member_handler::{create_member, delete_member, list_members, update_member},
    stats_handler::{get_admin_stats, get_siswa_stats},
    transaction_handler::{borrow_book, delete_transaction, list_transactions, return_book},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup logger/tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "perpustakaan_ukk=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Inisialisasi Database SQLite
    println!("🚀 Menginisialisasi Database SQLite...");
    let db_pool = match db::init_db() {
        Ok(pool) => {
            println!("✅ Basis data 'perpustakaan.db' siap digunakan!");
            pool
        }
        Err(e) => {
            eprintln!("❌ Gagal menghubungkan basis data: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Konfigurasi CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 4. API Routes
    let api_routes = Router::new()
        // Auth
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/me", get(get_me))
        .route("/auth/logout", post(logout))
        // Books & Categories
        .route("/books", get(list_books).post(create_book))
        .route("/books/:id", get(get_book_detail).put(update_book).delete(delete_book))
        .route("/categories", get(list_categories).post(create_category))
        // Members (Admin only)
        .route("/members", get(list_members).post(create_member))
        .route("/members/:id", put(update_member).delete(delete_member))
        // Transactions
        .route("/transactions", get(list_transactions))
        .route("/transactions/borrow", post(borrow_book))
        .route("/transactions/return", post(return_book))
        .route("/transactions/:id", delete(delete_transaction))
        // Stats
        .route("/stats/admin", get(get_admin_stats))
        .route("/stats/siswa", get(get_siswa_stats))
        .with_state(db_pool.clone());

    // 5. Static Files Serving
    let static_service = ServeDir::new("static")
        .append_index_html_on_directories(true);

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(static_service)
        .layer(cors);

    // 6. Bind Socket Address (Utama: 3000, Alternatif: 3001)
    let port = 3000;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    // Jalankan juga di port alternatif 3001 untuk menghindari cache browser dari proyek lama
    let app_alt = app.clone();
    tokio::spawn(async move {
        let addr_alt = SocketAddr::from(([0, 0, 0, 0], 3001));
        if let Ok(listener_alt) = tokio::net::TcpListener::bind(addr_alt).await {
            println!("   Port Alternatif aktif di: http://localhost:3001");
            let _ = axum::serve(listener_alt, app_alt).await;
        }
    });

    println!("\n=======================================================");
    println!("   PERPUSTAKAAN SEKOLAH DIGITAL - UKK RPL 2025/2026");
    println!("   Backend : Rust + Axum + Tokio + SQLite");
    println!("   Aplikasi berjalan di: http://localhost:{}", port);
    println!("   Akses Alternatif    : http://localhost:3001");
    println!("   Akun Default:");
    println!("     - Admin : username = admin  | password = admin123");
    println!("     - Siswa : username = siswa1 | password = siswa123");
    println!("=======================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
