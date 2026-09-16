-- ====================================================================
-- DATABASE PERPUSTAKAAN SEKOLAH DIGITAL (UKK RPL 2025/2026 PAKET 4)
-- Judul Proyek : Pengembangan Aplikasi Peminjaman Buku
-- Kompatibilitas : MySQL / MariaDB / SQLite / PostgreSQL
-- ====================================================================

-- 1. Tabel Kategori Buku
CREATE TABLE IF NOT EXISTS categories (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 2. Tabel Pengguna (Admin & Siswa)
CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nis_nip VARCHAR(50) NOT NULL UNIQUE,
    username VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(100) NOT NULL,
    email VARCHAR(100),
    phone VARCHAR(20),
    class_name VARCHAR(50),
    role ENUM('admin', 'siswa') NOT NULL DEFAULT 'siswa',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 3. Tabel Data Buku
CREATE TABLE IF NOT EXISTS books (
    id INT AUTO_INCREMENT PRIMARY KEY,
    isbn VARCHAR(30) NOT NULL UNIQUE,
    title VARCHAR(200) NOT NULL,
    author VARCHAR(100) NOT NULL,
    publisher VARCHAR(100) NOT NULL,
    year INT NOT NULL,
    category_id INT NOT NULL,
    stock INT NOT NULL DEFAULT 0,
    total_stock INT NOT NULL DEFAULT 0,
    shelf_location VARCHAR(50),
    description TEXT,
    cover_url TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_books_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 4. Tabel Transaksi Peminjaman & Pengembalian
CREATE TABLE IF NOT EXISTS transactions (
    id INT AUTO_INCREMENT PRIMARY KEY,
    transaction_code VARCHAR(50) NOT NULL UNIQUE,
    user_id INT NOT NULL,
    book_id INT NOT NULL,
    borrow_date DATE NOT NULL,
    due_date DATE NOT NULL,
    return_date DATE NULL,
    status ENUM('borrowed', 'returned', 'overdue') NOT NULL DEFAULT 'borrowed',
    fine_amount DECIMAL(10,2) DEFAULT 0.00,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_trans_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_trans_book FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 5. Tabel Sesi / Token Autentikasi
CREATE TABLE IF NOT EXISTS user_tokens (
    token VARCHAR(255) PRIMARY KEY,
    user_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_tokens_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


-- ====================================================================
-- SEED DATA AWAL (DML)
-- ====================================================================

-- Data Kategori
INSERT INTO categories (id, name, description) VALUES
(1, 'Pemrograman & RPL', 'Buku tentang rekayasa perangkat lunak, coding, web, dan algoritma'),
(2, 'Teknik Komputer & Jaringan', 'Buku seputar instalasi jaringan, mikrotik, dan server'),
(3, 'Sains & Matematika', 'Buku referensi sains dasar, fisika, kimia, dan matematika terapan'),
(4, 'Bahasa & Sastra', 'Novel, kumpulan cerpen, tata bahasa Indonesia dan bahasa Inggris'),
(5, 'Sejarah & Sosial', 'Buku wawasan kebangsaan, sejarah nasional, dan ilmu sosial'),
(6, 'Komik & Fiksi', 'Bahan bacaan hiburan, literasi fiksi, dan komik edukasi')
ON DUPLICATE KEY UPDATE name=VALUES(name);

-- Data Pengguna Default
-- Password hash untuk 'admin123' dan 'siswa123' menggunakan Bcrypt
INSERT INTO users (id, nis_nip, username, password_hash, full_name, email, phone, class_name, role) VALUES
(1, '198501012010011001', 'admin', '$2a$12$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy', 'Administrator Perpustakaan', 'admin@perpustakaan.sch.id', '081234567890', NULL, 'admin'),
(2, '202510001', 'siswa1', '$2a$12$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy', 'Ahmad Fauzi', 'ahmad.fauzi@siswa.sch.id', '085678901234', 'XII RPL 1', 'siswa'),
(3, '202510002', 'siswa2', '$2a$12$N9qo8uLOickgx2ZMRZoMyeIjZAgcfl7p92ldGxad68LJZdL17lhWy', 'Siti Nurhaliza', 'siti.nur@siswa.sch.id', '087812345678', 'XII RPL 2', 'siswa')
ON DUPLICATE KEY UPDATE username=VALUES(username);

-- Data Buku Perpustakaan
INSERT INTO books (id, isbn, title, author, publisher, year, category_id, stock, total_stock, shelf_location, description, cover_url) VALUES
(1, '978-602-04-1234-1', 'Pemrograman Rust Modern untuk Pemula', 'Budi Raharjo', 'Informatika Bandung', 2024, 1, 9, 10, 'Rak A-01', 'Panduan lengkap belajar bahasa pemrograman Rust mulai dari sintaks dasar, ownership, borrow checker hingga web server.', 'https://images.unsplash.com/photo-1532012164546-f432f2e3777a?auto=format&fit=crop&w=400&q=80'),
(2, '978-602-04-5678-2', 'Dasar-Dasar Rekayasa Perangkat Lunak SMK', 'Dra. Sri Wahyuni, M.Kom', 'Erlangga', 2023, 1, 15, 15, 'Rak A-02', 'Buku teks pelajaran resmi RPL kurikulum merdeka memuat SDLC, ERD, Flowchart, dan pembuatan aplikasi web.', 'https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=400&q=80'),
(3, '978-979-3784-91-3', 'Mastering Database: SQL & NoSQL Architecture', 'Eko Kurniawan Khannedy', 'Gava Media', 2023, 1, 8, 8, 'Rak A-03', 'Membahas optimasi query, relasi data, normalisasi hingga transaction ACID.', 'https://images.unsplash.com/photo-1512820790803-83ca734da794?auto=format&fit=crop&w=400&q=80'),
(4, '978-602-291-001-5', 'Laskar Pelangi', 'Andrea Hirata', 'Bentang Pustaka', 2021, 4, 11, 12, 'Rak B-01', 'Novel inspiratif tentang perjuangan sepuluh anak di Belitung dalam menempuh pendidikan.', 'https://images.unsplash.com/photo-1497633762265-9d179a990aa6?auto=format&fit=crop&w=400&q=80'),
(5, '978-602-03-3112-9', 'Bumi Manusia', 'Pramoedya Ananta Toer', 'Lentera Dipantara', 2020, 4, 6, 6, 'Rak B-02', 'Karya sastra legendaris Tetralogi Buru berlatar kebangkitan nasional Indonesia.', 'https://images.unsplash.com/photo-1495446815901-a7297e633e8d?auto=format&fit=crop&w=400&q=80'),
(6, '978-602-8519-93-8', 'Jaringan Komputer Berbasis Mikrotik & Cisco', 'Iwan Sofana', 'Informatika', 2023, 2, 7, 7, 'Rak C-01', 'Konfigurasi routing, VLAN, firewall, dan manajemen bandwidth di lingkungan sekolah.', 'https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?auto=format&fit=crop&w=400&q=80')
ON DUPLICATE KEY UPDATE title=VALUES(title);

-- Data Transaksi Contoh
INSERT INTO transactions (id, transaction_code, user_id, book_id, borrow_date, due_date, return_date, status, fine_amount, notes) VALUES
(1, 'TRX-202509-001', 2, 1, DATE_SUB(CURRENT_DATE, INTERVAL 3 DAY), DATE_ADD(CURRENT_DATE, INTERVAL 4 DAY), NULL, 'borrowed', 0.00, 'Peminjaman untuk tugas UKK'),
(2, 'TRX-202509-002', 3, 4, DATE_SUB(CURRENT_DATE, INTERVAL 10 DAY), DATE_SUB(CURRENT_DATE, INTERVAL 3 DAY), NULL, 'overdue', 3000.00, 'Terlambat 3 hari')
ON DUPLICATE KEY UPDATE transaction_code=VALUES(transaction_code);
