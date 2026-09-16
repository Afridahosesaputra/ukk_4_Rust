use std::sync::{Arc, Mutex};
use rusqlite::{Connection, Result};
use bcrypt::{hash, DEFAULT_COST};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db() -> Result<DbPool> {
    let conn = Connection::open("perpustakaan.db")?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // Create Tables
    conn.execute_batch(
        r#"
        -- 1. Table Users
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nis_nip VARCHAR(50) UNIQUE NOT NULL,
            username VARCHAR(50) UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            full_name VARCHAR(100) NOT NULL,
            email VARCHAR(100),
            phone VARCHAR(20),
            class_name VARCHAR(50),
            role VARCHAR(20) NOT NULL DEFAULT 'siswa', -- 'admin' atau 'siswa'
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 2. Table Categories
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name VARCHAR(100) UNIQUE NOT NULL,
            description TEXT
        );

        -- 3. Table Books
        CREATE TABLE IF NOT EXISTS books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            isbn VARCHAR(30) UNIQUE NOT NULL,
            title VARCHAR(200) NOT NULL,
            author VARCHAR(100) NOT NULL,
            publisher VARCHAR(100) NOT NULL,
            year INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            stock INTEGER NOT NULL DEFAULT 0,
            total_stock INTEGER NOT NULL DEFAULT 0,
            shelf_location VARCHAR(50),
            description TEXT,
            cover_url TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE RESTRICT
        );

        -- 4. Table Transactions
        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            transaction_code VARCHAR(50) UNIQUE NOT NULL,
            user_id INTEGER NOT NULL,
            book_id INTEGER NOT NULL,
            borrow_date DATE NOT NULL,
            due_date DATE NOT NULL,
            return_date DATE,
            status VARCHAR(20) NOT NULL DEFAULT 'borrowed', -- 'borrowed', 'returned', 'overdue'
            fine_amount REAL DEFAULT 0.0,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
            FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE RESTRICT
        );

        -- 5. Table Sessions / Tokens (simulasi otentikasi)
        CREATE TABLE IF NOT EXISTS user_tokens (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        );
        "#
    )?;

    // Seed default data if users table is empty
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    if user_count == 0 {
        seed_data(&conn)?;
    }

    Ok(Arc::new(Mutex::new(conn)))
}

fn seed_data(conn: &Connection) -> Result<()> {
    // Default Admin password: admin123
    let admin_hash = hash("admin123", DEFAULT_COST).unwrap_or_else(|_| "$2b$12$e8Y68O8...".to_string());
    // Default Siswa password: siswa123
    let siswa_hash = hash("siswa123", DEFAULT_COST).unwrap_or_else(|_| "$2b$12$e8Y68O8...".to_string());

    // Insert Admin
    conn.execute(
        "INSERT INTO users (nis_nip, username, password_hash, full_name, email, phone, role)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            "198501012010011001",
            "admin",
            &admin_hash,
            "Administrator Perpustakaan",
            "admin@perpustakaan.sch.id",
            "081234567890",
            "admin",
        ),
    )?;

    // Insert Siswa Default
    conn.execute(
        "INSERT INTO users (nis_nip, username, password_hash, full_name, email, phone, class_name, role)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            "202510001",
            "siswa1",
            &siswa_hash,
            "Ahmad Fauzi",
            "ahmad.fauzi@siswa.sch.id",
            "085678901234",
            "XII RPL 1",
            "siswa",
        ),
    )?;

    conn.execute(
        "INSERT INTO users (nis_nip, username, password_hash, full_name, email, phone, class_name, role)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            "202510002",
            "siswa2",
            &siswa_hash,
            "Siti Nurhaliza",
            "siti.nur@siswa.sch.id",
            "087812345678",
            "XII RPL 2",
            "siswa",
        ),
    )?;

    // Insert Categories
    let categories = [
        ("Pemrograman & RPL", "Buku tentang rekayasa perangkat lunak, coding, web, dan algoritma"),
        ("Teknik Komputer & Jaringan", "Buku seputar instalasi jaringan, mikrotik, dan server"),
        ("Sains & Matematika", "Buku referensi sains dasar, fisika, kimia, dan matematika terapan"),
        ("Bahasa & Sastra", "Novel, kumpulan cerpen, tata bahasa Indonesia dan bahasa Inggris"),
        ("Sejarah & Sosial", "Buku wawasan kebangsaan, sejarah nasional, dan ilmu sosial"),
        ("Komik & Fiksi", "Bahan bacaan hiburan, literasi fiksi, dan komik edukasi"),
    ];

    for (name, desc) in categories.iter() {
        conn.execute(
            "INSERT INTO categories (name, description) VALUES (?1, ?2)",
            (name, desc),
        )?;
    }

    // Insert Books
    let books = [
        (
            "978-602-04-1234-1",
            "Pemrograman Rust Modern untuk Pemula",
            "Budi Raharjo",
            "Informatika Bandung",
            2024,
            1,
            10,
            10,
            "Rak A-01",
            "Panduan lengkap belajar bahasa pemrograman Rust mulai dari sintaks dasar, ownership, borrow checker hingga web server.",
            "https://images.unsplash.com/photo-1532012164546-f432f2e3777a?auto=format&fit=crop&w=400&q=80",
        ),
        (
            "978-602-04-5678-2",
            "Dasar-Dasar Rekayasa Perangkat Lunak SMK",
            "Dra. Sri Wahyuni, M.Kom",
            "Erlangga",
            2023,
            1,
            15,
            15,
            "Rak A-02",
            "Buku teks pelajaran resmi RPL kurikulum merdeka memuat SDLC, ERD, Flowchart, dan pembuatan aplikasi web.",
            "https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=400&q=80",
        ),
        (
            "978-979-3784-91-3",
            "Mastering Database: SQL & NoSQL Architecture",
            "Eko Kurniawan Khannedy",
            "Gava Media",
            2023,
            1,
            8,
            8,
            "Rak A-03",
            "Membahas optimasi query, relasi data, normalisasi hingga transaction ACID.",
            "https://images.unsplash.com/photo-1512820790803-83ca734da794?auto=format&fit=crop&w=400&q=80",
        ),
        (
            "978-602-291-001-5",
            "Laskar Pelangi",
            "Andrea Hirata",
            "Bentang Pustaka",
            2021,
            4,
            12,
            12,
            "Rak B-01",
            "Novel inspiratif tentang perjuangan sepuluh anak di Belitung dalam menempuh pendidikan.",
            "https://images.unsplash.com/photo-1497633762265-9d179a990aa6?auto=format&fit=crop&w=400&q=80",
        ),
        (
            "978-602-03-3112-9",
            "Bumi Manusia",
            "Pramoedya Ananta Toer",
            "Lentera Dipantara",
            2020,
            4,
            6,
            6,
            "Rak B-02",
            "Karya sastra legendaris Tetralogi Buru berlatar kebangkitan nasional Indonesia.",
            "https://images.unsplash.com/photo-1495446815901-a7297e633e8d?auto=format&fit=crop&w=400&q=80",
        ),
        (
            "978-602-8519-93-8",
            "Jaringan Komputer Berbasis Mikrotik & Cisco",
            "Iwan Sofana",
            "Informatika",
            2023,
            2,
            7,
            7,
            "Rak C-01",
            "Konfigurasi routing, VLAN, firewall, dan manajemen bandwidth di lingkungan sekolah.",
            "https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?auto=format&fit=crop&w=400&q=80",
        ),
    ];

    for b in books.iter() {
        conn.execute(
            "INSERT INTO books (isbn, title, author, publisher, year, category_id, stock, total_stock, shelf_location, description, cover_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (b.0, b.1, b.2, b.3, b.4, b.5, b.6, b.7, b.8, b.9, b.10),
        )?;
    }

    // Insert Initial Transactions Sample (Siswa 1 pinjam Buku 1)
    conn.execute(
        "INSERT INTO transactions (transaction_code, user_id, book_id, borrow_date, due_date, status, fine_amount, notes)
         VALUES ('TRX-202509-001', 2, 1, date('now', '-3 day'), date('now', '+4 day'), 'borrowed', 0.0, 'Peminjaman untuk tugas UKK')",
        [],
    )?;
    // Kurangi stok buku 1 sebanyak 1
    conn.execute("UPDATE books SET stock = stock - 1 WHERE id = 1", [])?;

    // Siswa 2 pinjam Buku 4 (Sudah lewat tanggal kembali untuk contoh denda)
    conn.execute(
        "INSERT INTO transactions (transaction_code, user_id, book_id, borrow_date, due_date, status, fine_amount, notes)
         VALUES ('TRX-202509-002', 3, 4, date('now', '-10 day'), date('now', '-3 day'), 'overdue', 3000.0, 'Terlambat 3 hari')",
        [],
    )?;
    // Kurangi stok buku 4 sebanyak 1
    conn.execute("UPDATE books SET stock = stock - 1 WHERE id = 4", [])?;

    println!("✅ Database berhasil diinisialisasi dan data awal (seeder) berhasil dibuat!");
    Ok(())
}
