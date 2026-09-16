// Force unregister any lingering service workers from other projects on this origin
if ('serviceWorker' in navigator) {
    navigator.serviceWorker.getRegistrations().then(function(registrations) {
        for (let registration of registrations) {
            registration.unregister();
            console.log("Unregistered lingering Service Worker:", registration);
        }
    });
}

/**
 * PustakaDigital SMK - Frontend Application Logic
 * Full SPA with Fetch API, State Management, and Print System
 */

const API_BASE = '/api';

const app = {
    state: {
        token: localStorage.getItem('pustaka_token') || null,
        user: JSON.parse(localStorage.getItem('pustaka_user') || 'null'),
        currentView: 'catalog',
        books: [],
        categories: [],
        members: [],
        transactions: [],
        stats: null,
        searchQuery: '',
        selectedCategory: '',
        selectedStatus: 'all',
    },

    async init() {
        // Validate user session if token exists
        if (this.state.token) {
            await this.fetchCurrentUser();
        }
        await this.loadCategories();
        this.renderNav();
        this.navigate('catalog');
    },

    // ==========================================
    // AUTHENTICATION
    // ==========================================
    async fetchCurrentUser() {
        try {
            const res = await fetch(`${API_BASE}/auth/me`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            if (data.success) {
                this.state.user = data.data;
                localStorage.setItem('pustaka_user', JSON.stringify(data.data));
            } else {
                this.logout(false);
            }
        } catch (e) {
            console.error("Auth check error:", e);
        }
    },

    async login(username, password) {
        try {
            const res = await fetch(`${API_BASE}/auth/login`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password })
            });
            const data = await res.json();
            if (data.success) {
                this.state.token = data.token;
                this.state.user = data.user;
                localStorage.setItem('pustaka_token', data.token);
                localStorage.setItem('pustaka_user', JSON.stringify(data.user));
                this.closeAllModals();
                this.showToast(data.message, 'success');
                this.renderNav();
                this.navigate(this.state.user.role === 'admin' ? 'dashboard' : 'catalog');
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal terhubung ke server', 'error');
        }
    },

    async register(formData) {
        try {
            const res = await fetch(`${API_BASE}/auth/register`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(formData)
            });
            const data = await res.json();
            if (data.success) {
                this.showToast(data.message, 'success');
                this.showLoginModal(formData.username);
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal mendaftar', 'error');
        }
    },

    logout(notify = true) {
        if (this.state.token) {
            fetch(`${API_BASE}/auth/logout`, {
                method: 'POST',
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
        }
        this.state.token = null;
        this.state.user = null;
        localStorage.removeItem('pustaka_token');
        localStorage.removeItem('pustaka_user');
        this.renderNav();
        this.navigate('catalog');
        if (notify) this.showToast('Anda telah logout', 'warning');
    },

    // ==========================================
    // NAVIGATION & VIEW ROUTING
    // ==========================================
    renderNav() {
        const navLinks = document.getElementById('nav-links');
        const authArea = document.getElementById('nav-auth-area');
        const user = this.state.user;

        let linksHtml = `
            <button class="nav-btn ${this.state.currentView === 'catalog' ? 'active' : ''}" onclick="app.navigate('catalog')">
                <i class="fa-solid fa-book-open"></i> Katalog Buku
            </button>
        `;

        if (user) {
            if (user.role === 'admin') {
                linksHtml += `
                    <button class="nav-btn ${this.state.currentView === 'dashboard' ? 'active' : ''}" onclick="app.navigate('dashboard')">
                        <i class="fa-solid fa-chart-pie"></i> Dashboard
                    </button>
                    <button class="nav-btn ${this.state.currentView === 'books_admin' ? 'active' : ''}" onclick="app.navigate('books_admin')">
                        <i class="fa-solid fa-boxes-stacked"></i> Kelola Buku
                    </button>
                    <button class="nav-btn ${this.state.currentView === 'members_admin' ? 'active' : ''}" onclick="app.navigate('members_admin')">
                        <i class="fa-solid fa-users"></i> Kelola Anggota
                    </button>
                    <button class="nav-btn ${this.state.currentView === 'transactions_admin' ? 'active' : ''}" onclick="app.navigate('transactions_admin')">
                        <i class="fa-solid fa-receipt"></i> Transaksi
                    </button>
                `;
            } else {
                linksHtml += `
                    <button class="nav-btn ${this.state.currentView === 'dashboard' ? 'active' : ''}" onclick="app.navigate('dashboard')">
                        <i class="fa-solid fa-chart-simple"></i> Dashboard Siswa
                    </button>
                    <button class="nav-btn ${this.state.currentView === 'my_borrows' ? 'active' : ''}" onclick="app.navigate('my_borrows')">
                        <i class="fa-solid fa-clock-rotate-left"></i> Peminjaman Saya
                    </button>
                `;
            }

            authArea.innerHTML = `
                <div class="user-badge-menu">
                    <div class="user-avatar-mini">${user.full_name.charAt(0).toUpperCase()}</div>
                    <div class="user-info-text">
                        <span class="user-info-name">${user.full_name}</span>
                        <span class="user-info-role">${user.role === 'admin' ? 'Petugas Admin' : (user.class_name || 'Siswa')}</span>
                    </div>
                </div>
                <button class="btn btn-sm btn-outline" onclick="app.logout()" title="Keluar">
                    <i class="fa-solid fa-right-from-bracket"></i> Keluar
                </button>
            `;
        } else {
            authArea.innerHTML = `
                <button class="btn btn-primary" onclick="app.showLoginModal()">
                    <i class="fa-solid fa-user-lock"></i> Masuk / Daftar
                </button>
            `;
        }

        navLinks.innerHTML = linksHtml;
    },

    navigate(view) {
        this.state.currentView = view;
        this.renderNav();
        const content = document.getElementById('app-content');

        switch (view) {
            case 'catalog':
                this.renderCatalogView(content);
                break;
            case 'dashboard':
                this.renderDashboardView(content);
                break;
            case 'books_admin':
                this.renderBooksAdminView(content);
                break;
            case 'members_admin':
                this.renderMembersAdminView(content);
                break;
            case 'transactions_admin':
                this.renderTransactionsAdminView(content);
                break;
            case 'my_borrows':
                this.renderMyBorrowsView(content);
                break;
            default:
                this.renderCatalogView(content);
        }
    },

    // ==========================================
    // 1. VIEW: CATALOG (SISWA & GUEST & ADMIN)
    // ==========================================
    async renderCatalogView(container) {
        container.innerHTML = `
            <!-- Hero Banner -->
            <div class="hero-banner">
                <div class="hero-content">
                    <div class="hero-badge"><i class="fa-solid fa-sparkles"></i> Pustaka Sekolah Digital Modern</div>
                    <h1 class="hero-title">Temukan & Pinjam Buku Pelajaran Favoritmu</h1>
                    <p class="hero-desc">Layanan digital perpustakaan sekolah untuk kemudahan peminjaman, pengembalian mandiri, dan eksplorasi ribuan koleksi buku berkualitas.</p>
                    <div class="hero-actions">
                        ${!this.state.user ? `
                            <button class="btn btn-white" onclick="app.showLoginModal()">
                                <i class="fa-solid fa-right-to-bracket"></i> Masuk Sekarang
                            </button>
                            <button class="btn btn-outline" style="color: #fff; border-color: rgba(255,255,255,0.4);" onclick="app.showRegisterModal()">
                                <i class="fa-solid fa-user-plus"></i> Daftar Siswa Baru
                            </button>
                        ` : `
                            <button class="btn btn-white" onclick="app.navigate('dashboard')">
                                <i class="fa-solid fa-gauge-high"></i> Buka Dashboard Saya
                            </button>
                        `}
                    </div>
                </div>
                <div class="hero-decoration">
                    <i class="fa-solid fa-book-journal-whills"></i>
                </div>
            </div>

            <!-- Filter & Search Bar -->
            <div class="filter-toolbar">
                <div class="search-input-box">
                    <i class="fa-solid fa-magnifying-glass"></i>
                    <input type="text" id="catalog-search" placeholder="Cari judul buku, pengarang, penerbit, atau ISBN..." value="${this.state.searchQuery}" oninput="app.handleSearch(this.value)">
                </div>
                <select class="filter-select" id="catalog-category-filter" onchange="app.handleCategoryFilter(this.value)">
                    <option value="">Semua Kategori</option>
                    ${this.state.categories.map(c => `<option value="${c.id}" ${this.state.selectedCategory == c.id ? 'selected' : ''}>${c.name}</option>`).join('')}
                </select>
                <button class="btn btn-outline" onclick="app.resetFilter()">
                    <i class="fa-solid fa-arrow-rotate-left"></i> Reset
                </button>
            </div>

            <!-- Section Title -->
            <div class="section-header">
                <div class="section-title-wrap">
                    <h2>Koleksi Buku Perpustakaan</h2>
                    <p>Pilih buku yang ingin dipinjam dan cek ketersediaan stok</p>
                </div>
                ${this.state.user && this.state.user.role === 'admin' ? `
                    <button class="btn btn-primary" onclick="app.showAddBookModal()">
                        <i class="fa-solid fa-plus"></i> Tambah Buku Baru
                    </button>
                ` : ''}
            </div>

            <!-- Books Grid Container -->
            <div id="books-grid" class="book-grid">
                <div style="grid-column: 1/-1; text-align: center; padding: 3rem;">
                    <i class="fa-solid fa-spinner fa-spin fa-2x" style="color: var(--primary);"></i>
                    <p style="margin-top: 1rem; color: var(--text-muted);">Memuat koleksi buku...</p>
                </div>
            </div>
        `;

        await this.loadBooks();
    },

    async loadBooks() {
        try {
            let url = `${API_BASE}/books?`;
            if (this.state.searchQuery) url += `q=${encodeURIComponent(this.state.searchQuery)}&`;
            if (this.state.selectedCategory) url += `category_id=${this.state.selectedCategory}&`;

            const res = await fetch(url);
            const data = await res.json();
            if (data.success) {
                this.state.books = data.data;
                this.renderBooksGrid();
            }
        } catch (e) {
            console.error("Load books error:", e);
        }
    },

    renderBooksGrid() {
        const grid = document.getElementById('books-grid');
        if (!grid) return;

        if (this.state.books.length === 0) {
            grid.innerHTML = `
                <div style="grid-column: 1/-1; text-align: center; padding: 4rem 1rem; background: #fff; border-radius: var(--radius-lg); border: 1px dashed var(--border-color);">
                    <i class="fa-solid fa-box-open fa-3x" style="color: var(--text-light); margin-bottom: 1rem;"></i>
                    <h3 style="color: var(--text-main);">Tidak ada buku yang sesuai</h3>
                    <p style="color: var(--text-muted); font-size: 0.9rem;">Coba cari dengan kata kunci lain atau pilih kategori yang berbeda.</p>
                </div>
            `;
            return;
        }

        grid.innerHTML = this.state.books.map(b => {
            const isAvailable = b.stock > 0;
            const defaultCover = 'https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=400&q=80';
            const cover = b.cover_url && b.cover_url.trim() !== '' ? b.cover_url : defaultCover;

            return `
                <div class="book-card">
                    <div class="book-cover-container">
                        <img class="book-cover-img" src="${cover}" alt="${b.title}" onerror="this.src='${defaultCover}'">
                        <span class="book-category-pill">${b.category_name || 'Umum'}</span>
                        <span class="book-stock-badge ${isAvailable ? 'stock-available' : 'stock-empty'}">
                            ${isAvailable ? `<i class="fa-solid fa-check"></i> Stok: ${b.stock}` : '<i class="fa-solid fa-xmark"></i> Habis'}
                        </span>
                    </div>
                    <div class="book-content">
                        <span class="book-meta-isbn">ISBN: ${b.isbn} &bull; ${b.year}</span>
                        <h3 class="book-title" title="${b.title}">${b.title}</h3>
                        <p class="book-author"><i class="fa-solid fa-feather-pointed"></i> ${b.author}</p>
                        ${b.shelf_location ? `<span class="book-shelf"><i class="fa-solid fa-location-dot"></i> ${b.shelf_location}</span>` : ''}
                        
                        <div class="book-footer">
                            <span style="font-size: 0.78rem; color: var(--text-muted);">Penerbit: <strong>${b.publisher}</strong></span>
                            ${isAvailable ? `
                                <button class="btn btn-sm btn-primary" onclick="app.showBorrowModal(${b.id})">
                                    <i class="fa-solid fa-hand-holding-hand"></i> Pinjam
                                </button>
                            ` : `
                                <button class="btn btn-sm btn-secondary" disabled>
                                    <i class="fa-solid fa-ban"></i> Habis
                                </button>
                            `}
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    },

    handleSearch(val) {
        this.state.searchQuery = val;
        clearTimeout(this._searchTimer);
        this._searchTimer = setTimeout(() => this.loadBooks(), 300);
    },

    handleCategoryFilter(val) {
        this.state.selectedCategory = val;
        this.loadBooks();
    },

    resetFilter() {
        this.state.searchQuery = '';
        this.state.selectedCategory = '';
        const s = document.getElementById('catalog-search');
        const c = document.getElementById('catalog-category-filter');
        if (s) s.value = '';
        if (c) c.value = '';
        this.loadBooks();
    },

    // ==========================================
    // 2. VIEW: DASHBOARD (ADMIN & SISWA)
    // ==========================================
    async renderDashboardView(container) {
        if (!this.state.user) {
            this.showLoginModal();
            return;
        }

        const isAdmin = this.state.user.role === 'admin';
        container.innerHTML = `
            <div class="section-header">
                <div class="section-title-wrap">
                    <h2>Dashboard ${isAdmin ? 'Administrator Perpustakaan' : 'Siswa'}</h2>
                    <p>Ringkasan statistik peminjaman dan aktivitas perpustakaan</p>
                </div>
                <div>
                    ${isAdmin ? `
                        <button class="btn btn-outline" onclick="app.printReport()">
                            <i class="fa-solid fa-print"></i> Cetak Laporan
                        </button>
                        <button class="btn btn-primary" onclick="app.showAddBookModal()">
                            <i class="fa-solid fa-plus"></i> Tambah Buku
                        </button>
                    ` : `
                        <button class="btn btn-primary" onclick="app.navigate('catalog')">
                            <i class="fa-solid fa-book-open"></i> Cari & Pinjam Buku
                        </button>
                    `}
                </div>
            </div>

            <!-- Stats Grid -->
            <div id="dashboard-stats" class="stats-grid">
                <div style="grid-column: 1/-1; text-align: center; padding: 2rem;">
                    <i class="fa-solid fa-spinner fa-spin fa-2x" style="color: var(--primary);"></i>
                </div>
            </div>

            <!-- Recent Table -->
            <div class="section-header" style="margin-top: 2rem;">
                <div class="section-title-wrap">
                    <h2>${isAdmin ? 'Transaksi Peminjaman Terbaru' : 'Status Peminjaman Aktif Anda'}</h2>
                    <p>${isAdmin ? '5 data transaksi peminjaman buku terakhir' : 'Daftar buku yang sedang Anda pinjam dan tanggal jatuh tempo'}</p>
                </div>
                <button class="btn btn-sm btn-outline" onclick="app.navigate('${isAdmin ? 'transactions_admin' : 'my_borrows'}')">
                    Lihat Semua <i class="fa-solid fa-arrow-right"></i>
                </button>
            </div>

            <div id="dashboard-recent-table" class="card-table-wrap">
                <div style="text-align: center; padding: 2rem;">Memuat data transaksi...</div>
            </div>
        `;

        await this.loadStats(isAdmin);
        await this.loadRecentDashboardTransactions(isAdmin);
    },

    async loadStats(isAdmin) {
        try {
            const url = isAdmin ? `${API_BASE}/stats/admin` : `${API_BASE}/stats/siswa`;
            const res = await fetch(url, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            if (data.success) {
                const s = data.data;
                const container = document.getElementById('dashboard-stats');
                if (!container) return;

                if (isAdmin) {
                    container.innerHTML = `
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-indigo">
                                <i class="fa-solid fa-book"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Total Judul Buku</span>
                                <span class="stat-value">${s.total_books}</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-blue">
                                <i class="fa-solid fa-cubes-stacked"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Total Eksemplar</span>
                                <span class="stat-value">${s.total_stock}</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-green">
                                <i class="fa-solid fa-users"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Anggota Siswa</span>
                                <span class="stat-value">${s.total_members}</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-amber">
                                <i class="fa-solid fa-hand-holding"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Sedang Dipinjam</span>
                                <span class="stat-value">${s.active_borrows}</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-rose">
                                <i class="fa-solid fa-triangle-exclamation"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Terlambat & Denda</span>
                                <span class="stat-value">Rp ${Number(s.total_fines).toLocaleString('id-ID')}</span>
                            </div>
                        </div>
                    `;
                } else {
                    container.innerHTML = `
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-indigo">
                                <i class="fa-solid fa-book-bookmark"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Sedang Dipinjam</span>
                                <span class="stat-value">${s.active_borrows} Buku</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-green">
                                <i class="fa-solid fa-circle-check"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Selesai Dikembalikan</span>
                                <span class="stat-value">${s.total_returned} Buku</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-amber">
                                <i class="fa-solid fa-triangle-exclamation"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Lewat Jatuh Tempo</span>
                                <span class="stat-value">${s.overdue_count} Buku</span>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon-wrapper stat-icon-rose">
                                <i class="fa-solid fa-money-bill-wave"></i>
                            </div>
                            <div class="stat-details">
                                <span class="stat-title">Estimasi Denda</span>
                                <span class="stat-value">Rp ${Number(s.total_fines).toLocaleString('id-ID')}</span>
                            </div>
                        </div>
                    `;
                }
            }
        } catch (e) {
            console.error("Load stats error:", e);
        }
    },

    async loadRecentDashboardTransactions(isAdmin) {
        try {
            const res = await fetch(`${API_BASE}/transactions`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const container = document.getElementById('dashboard-recent-table');
            if (!container) return;

            if (data.success && data.data) {
                const list = data.data.slice(0, 5);
                if (list.length === 0) {
                    container.innerHTML = `
                        <div style="text-align: center; padding: 2.5rem; color: var(--text-muted);">
                            Belum ada riwayat transaksi peminjaman buku.
                        </div>
                    `;
                    return;
                }

                container.innerHTML = `
                    <div class="table-responsive">
                        <table class="custom-table">
                            <thead>
                                <tr>
                                    <th>Kode Transaksi</th>
                                    ${isAdmin ? '<th>Peminjam (NIS)</th>' : ''}
                                    <th>Judul Buku</th>
                                    <th>Tgl Pinjam</th>
                                    <th>Jatuh Tempo</th>
                                    <th>Status</th>
                                    <th>Denda</th>
                                    <th style="text-align: right;">Aksi</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${list.map(t => `
                                    <tr>
                                        <td><strong>${t.transaction_code}</strong></td>
                                        ${isAdmin ? `<td>${t.user_name || '-'} <small class="text-muted">(${t.nis_nip || '-'})</small></td>` : ''}
                                        <td>${t.book_title || '-'}</td>
                                        <td>${t.borrow_date}</td>
                                        <td>${t.due_date}</td>
                                        <td>
                                            <span class="badge badge-${t.status}">
                                                ${t.status === 'borrowed' ? 'Dipinjam' : (t.status === 'returned' ? 'Kembali' : 'Terlambat')}
                                            </span>
                                        </td>
                                        <td>${t.fine_amount > 0 ? `<strong style="color: var(--danger);">Rp ${Number(t.fine_amount).toLocaleString('id-ID')}</strong>` : '-'}</td>
                                        <td style="text-align: right;">
                                            ${t.status !== 'returned' ? `
                                                <button class="btn btn-sm btn-success" onclick="app.showReturnModal(${t.id}, '${t.book_title}', ${t.fine_amount})">
                                                    <i class="fa-solid fa-rotate-left"></i> Kembalikan
                                                </button>
                                            ` : '<span style="color: var(--success); font-size: 0.8rem;"><i class="fa-solid fa-check"></i> Selesai</span>'}
                                        </td>
                                    </tr>
                                `).join('')}
                            </tbody>
                        </table>
                    </div>
                `;
            }
        } catch (e) {
            console.error("Load recent transactions error:", e);
        }
    },

    // ==========================================
    // 3. VIEW: KELOLA BUKU (ADMIN)
    // ==========================================
    async renderBooksAdminView(container) {
        if (!this.state.user || this.state.user.role !== 'admin') {
            this.navigate('catalog');
            return;
        }

        container.innerHTML = `
            <div class="section-header">
                <div class="section-title-wrap">
                    <h2>Kelola Data Buku</h2>
                    <p>Manajemen data buku perpustakaan, nomor ISBN, kategori, dan stok</p>
                </div>
                <div style="display: flex; gap: 0.5rem;">
                    <button class="btn btn-outline" onclick="app.showAddCategoryModal()">
                        <i class="fa-solid fa-tags"></i> Kategori
                    </button>
                    <button class="btn btn-primary" onclick="app.showAddBookModal()">
                        <i class="fa-solid fa-plus"></i> Tambah Buku
                    </button>
                </div>
            </div>

            <div class="filter-toolbar">
                <div class="search-input-box">
                    <i class="fa-solid fa-magnifying-glass"></i>
                    <input type="text" id="admin-book-search" placeholder="Cari ISBN, judul, atau pengarang..." oninput="app.loadAdminBooksTable(this.value)">
                </div>
            </div>

            <div id="admin-books-table-wrap" class="card-table-wrap">
                <div style="text-align: center; padding: 2rem;">Memuat daftar buku...</div>
            </div>
        `;

        await this.loadAdminBooksTable('');
    },

    async loadAdminBooksTable(query = '') {
        try {
            const res = await fetch(`${API_BASE}/books?q=${encodeURIComponent(query)}`);
            const data = await res.json();
            const wrap = document.getElementById('admin-books-table-wrap');
            if (!wrap) return;

            if (data.success && data.data) {
                if (data.data.length === 0) {
                    wrap.innerHTML = `<div style="text-align: center; padding: 2.5rem; color: var(--text-muted);">Tidak ada data buku.</div>`;
                    return;
                }

                wrap.innerHTML = `
                    <div class="table-responsive">
                        <table class="custom-table">
                            <thead>
                                <tr>
                                    <th>Cover</th>
                                    <th>ISBN / Judul Buku</th>
                                    <th>Pengarang & Penerbit</th>
                                    <th>Kategori</th>
                                    <th>Tahun</th>
                                    <th>Stok Tersedia</th>
                                    <th>Lokasi Rak</th>
                                    <th style="text-align: right;">Aksi</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${data.data.map(b => {
                                    const defaultCover = 'https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=400&q=80';
                                    const cover = b.cover_url || defaultCover;
                                    return `
                                        <tr>
                                            <td style="width: 50px;">
                                                <img src="${cover}" alt="cover" style="width: 42px; height: 56px; object-fit: cover; border-radius: 4px;" onerror="this.src='${defaultCover}'">
                                            </td>
                                            <td>
                                                <strong style="color: var(--text-main); font-size: 0.95rem;">${b.title}</strong><br>
                                                <small style="color: var(--text-light); font-weight: 600;">ISBN: ${b.isbn}</small>
                                            </td>
                                            <td>${b.author}<br><small style="color: var(--text-muted);">${b.publisher}</small></td>
                                            <td><span class="badge" style="background: #f1f5f9; color: var(--text-main);">${b.category_name || '-'}</span></td>
                                            <td>${b.year}</td>
                                            <td>
                                                <span class="badge ${b.stock > 0 ? 'stock-available' : 'stock-empty'}" style="position: static;">
                                                    ${b.stock} / ${b.total_stock}
                                                </span>
                                            </td>
                                            <td><small class="book-shelf" style="margin: 0;">${b.shelf_location || '-'}</small></td>
                                            <td style="text-align: right;">
                                                <button class="btn btn-sm btn-outline" onclick="app.showEditBookModal(${b.id})" title="Edit Buku">
                                                    <i class="fa-solid fa-pen-to-square"></i>
                                                </button>
                                                <button class="btn btn-sm btn-danger" onclick="app.deleteBook(${b.id}, '${b.title}')" title="Hapus Buku">
                                                    <i class="fa-solid fa-trash-can"></i>
                                                </button>
                                            </td>
                                        </tr>
                                    `;
                                }).join('')}
                            </tbody>
                        </table>
                    </div>
                `;
            }
        } catch (e) {
            console.error("Load admin books error:", e);
        }
    },

    // ==========================================
    // 4. VIEW: KELOLA ANGGOTA (ADMIN)
    // ==========================================
    async renderMembersAdminView(container) {
        if (!this.state.user || this.state.user.role !== 'admin') {
            this.navigate('catalog');
            return;
        }

        container.innerHTML = `
            <div class="section-header">
                <div class="section-title-wrap">
                    <h2>Kelola Anggota & Siswa</h2>
                    <p>Manajemen pendaftaran siswa, NIS, kelas, dan reset password</p>
                </div>
                <button class="btn btn-primary" onclick="app.showAddMemberModal()">
                    <i class="fa-solid fa-user-plus"></i> Tambah Anggota Baru
                </button>
            </div>

            <div class="filter-toolbar">
                <div class="search-input-box">
                    <i class="fa-solid fa-magnifying-glass"></i>
                    <input type="text" id="admin-member-search" placeholder="Cari NIS, nama anggota, atau username..." oninput="app.loadAdminMembersTable(this.value)">
                </div>
            </div>

            <div id="admin-members-table-wrap" class="card-table-wrap">
                <div style="text-align: center; padding: 2rem;">Memuat data anggota...</div>
            </div>
        `;

        await this.loadAdminMembersTable('');
    },

    async loadAdminMembersTable(query = '') {
        try {
            const res = await fetch(`${API_BASE}/members?q=${encodeURIComponent(query)}`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const wrap = document.getElementById('admin-members-table-wrap');
            if (!wrap) return;

            if (data.success && data.data) {
                if (data.data.length === 0) {
                    wrap.innerHTML = `<div style="text-align: center; padding: 2.5rem; color: var(--text-muted);">Tidak ada data anggota.</div>`;
                    return;
                }

                wrap.innerHTML = `
                    <div class="table-responsive">
                        <table class="custom-table">
                            <thead>
                                <tr>
                                    <th>NIS / NIP</th>
                                    <th>Nama Lengkap</th>
                                    <th>Username</th>
                                    <th>Kelas</th>
                                    <th>Kontak / Email</th>
                                    <th>Peran</th>
                                    <th style="text-align: right;">Aksi</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${data.data.map(m => `
                                    <tr>
                                        <td><strong>${m.nis_nip}</strong></td>
                                        <td>
                                            <strong>${m.full_name}</strong>
                                        </td>
                                        <td><code>@${m.username}</code></td>
                                        <td>${m.class_name ? `<span class="badge" style="background:#e0f2fe; color:#0369a1;">${m.class_name}</span>` : '-'}</td>
                                        <td>${m.phone || '-'}<br><small style="color:var(--text-muted);">${m.email || '-'}</small></td>
                                        <td>
                                            <span class="badge badge-role-${m.role}">
                                                ${m.role === 'admin' ? '<i class="fa-solid fa-shield-halved"></i> Admin' : '<i class="fa-solid fa-user-graduate"></i> Siswa'}
                                            </span>
                                        </td>
                                        <td style="text-align: right;">
                                            <button class="btn btn-sm btn-outline" onclick="app.showEditMemberModal(${m.id})" title="Edit Anggota">
                                                <i class="fa-solid fa-user-pen"></i>
                                            </button>
                                            ${m.role !== 'admin' ? `
                                                <button class="btn btn-sm btn-danger" onclick="app.deleteMember(${m.id}, '${m.full_name}')" title="Hapus Anggota">
                                                    <i class="fa-solid fa-trash-can"></i>
                                                </button>
                                            ` : ''}
                                        </td>
                                    </tr>
                                `).join('')}
                            </tbody>
                        </table>
                    </div>
                `;
            }
        } catch (e) {
            console.error("Load members error:", e);
        }
    },

    // ==========================================
    // 5. VIEW: KELOLA TRANSAKSI (ADMIN)
    // ==========================================
    async renderTransactionsAdminView(container) {
        if (!this.state.user || this.state.user.role !== 'admin') {
            this.navigate('catalog');
            return;
        }

        container.innerHTML = `
            <div class="section-header">
                <div class="section-title-wrap">
                    <h2>Kelola Transaksi Peminjaman & Pengembalian</h2>
                    <p>Semua catatan peminjaman buku seluruh siswa, kalkulasi denda, dan laporan</p>
                </div>
                <button class="btn btn-primary" onclick="app.printReport()">
                    <i class="fa-solid fa-print"></i> Cetak Laporan PDF
                </button>
            </div>

            <div class="filter-toolbar">
                <div class="search-input-box">
                    <i class="fa-solid fa-magnifying-glass"></i>
                    <input type="text" id="admin-trx-search" placeholder="Cari kode transaksi, nama siswa, NIS, atau judul buku..." oninput="app.loadAdminTransactionsTable()">
                </div>
                <select class="filter-select" id="admin-trx-status-filter" onchange="app.loadAdminTransactionsTable()">
                    <option value="all">Semua Status</option>
                    <option value="borrowed">Sedang Dipinjam</option>
                    <option value="returned">Sudah Dikembalikan</option>
                    <option value="overdue">Terlambat</option>
                </select>
            </div>

            <div id="admin-transactions-table-wrap" class="card-table-wrap">
                <div style="text-align: center; padding: 2rem;">Memuat data transaksi...</div>
            </div>
        `;

        await this.loadAdminTransactionsTable();
    },

    async loadAdminTransactionsTable() {
        try {
            const searchInput = document.getElementById('admin-trx-search');
            const statusFilter = document.getElementById('admin-trx-status-filter');
            const q = searchInput ? searchInput.value : '';
            const status = statusFilter ? statusFilter.value : 'all';

            const res = await fetch(`${API_BASE}/transactions?q=${encodeURIComponent(q)}&status=${status}`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const wrap = document.getElementById('admin-transactions-table-wrap');
            if (!wrap) return;

            if (data.success && data.data) {
                this.state.transactions = data.data;
                if (data.data.length === 0) {
                    wrap.innerHTML = `<div style="text-align: center; padding: 2.5rem; color: var(--text-muted);">Tidak ada data transaksi.</div>`;
                    return;
                }

                wrap.innerHTML = `
                    <div class="table-responsive">
                        <table class="custom-table">
                            <thead>
                                <tr>
                                    <th>Kode TRX</th>
                                    <th>Peminjam (Siswa)</th>
                                    <th>Judul Buku</th>
                                    <th>Tgl Pinjam</th>
                                    <th>Jatuh Tempo</th>
                                    <th>Tgl Kembali</th>
                                    <th>Status</th>
                                    <th>Denda</th>
                                    <th style="text-align: right;">Aksi</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${data.data.map(t => `
                                    <tr>
                                        <td><code>${t.transaction_code}</code></td>
                                        <td>
                                            <strong>${t.user_name || '-'}</strong><br>
                                            <small style="color:var(--text-muted);">NIS: ${t.nis_nip || '-'}</small>
                                        </td>
                                        <td>
                                            <strong>${t.book_title || '-'}</strong><br>
                                            <small style="color:var(--text-light);">ISBN: ${t.book_isbn || '-'}</small>
                                        </td>
                                        <td>${t.borrow_date}</td>
                                        <td><strong>${t.due_date}</strong></td>
                                        <td>${t.return_date || '-'}</td>
                                        <td>
                                            <span class="badge badge-${t.status}">
                                                ${t.status === 'borrowed' ? 'Dipinjam' : (t.status === 'returned' ? 'Kembali' : 'Terlambat')}
                                            </span>
                                        </td>
                                        <td>
                                            ${t.fine_amount > 0 ? `<span style="color: var(--danger); font-weight: 700;">Rp ${Number(t.fine_amount).toLocaleString('id-ID')}</span>` : '-'}
                                        </td>
                                        <td style="text-align: right;">
                                            ${t.status !== 'returned' ? `
                                                <button class="btn btn-sm btn-success" onclick="app.showReturnModal(${t.id}, '${t.book_title}', ${t.fine_amount})" title="Proses Pengembalian">
                                                    <i class="fa-solid fa-rotate-left"></i> Kembali
                                                </button>
                                            ` : ''}
                                            <button class="btn btn-sm btn-outline" onclick="app.deleteTransaction(${t.id})" title="Hapus Riwayat">
                                                <i class="fa-solid fa-trash-can"></i>
                                            </button>
                                        </td>
                                    </tr>
                                `).join('')}
                            </tbody>
                        </table>
                    </div>
                `;
            }
        } catch (e) {
            console.error("Load admin transactions error:", e);
        }
    },

    // ==========================================
    // 6. VIEW: PEMINJAMAN SAYA (SISWA)
    // ==========================================
    async renderMyBorrowsView(container) {
        if (!this.state.user) {
            this.showLoginModal();
            return;
        }

        container.innerHTML = `
            <div class="section-header">
                <div class="section-title-wrap">
                    <h2>Riwayat Peminjaman Buku Saya</h2>
                    <p>Daftar buku yang pernah dan sedang Anda pinjam di perpustakaan</p>
                </div>
                <button class="btn btn-primary" onclick="app.navigate('catalog')">
                    <i class="fa-solid fa-plus"></i> Pinjam Buku Baru
                </button>
            </div>

            <div id="my-borrows-table-wrap" class="card-table-wrap">
                <div style="text-align: center; padding: 2rem;">Memuat riwayat peminjaman...</div>
            </div>
        `;

        try {
            const res = await fetch(`${API_BASE}/transactions`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const wrap = document.getElementById('my-borrows-table-wrap');
            if (!wrap) return;

            if (data.success && data.data) {
                if (data.data.length === 0) {
                    wrap.innerHTML = `
                        <div style="text-align: center; padding: 3rem; color: var(--text-muted);">
                            <i class="fa-solid fa-book fa-3x" style="color: var(--text-light); margin-bottom: 1rem;"></i>
                            <h3 style="color: var(--text-main);">Belum ada riwayat peminjaman</h3>
                            <p style="margin-top: 0.5rem;">Silakan jelajahi katalog untuk mulai meminjam buku.</p>
                        </div>
                    `;
                    return;
                }

                wrap.innerHTML = `
                    <div class="table-responsive">
                        <table class="custom-table">
                            <thead>
                                <tr>
                                    <th>Kode Transaksi</th>
                                    <th>Judul Buku</th>
                                    <th>Tgl Pinjam</th>
                                    <th>Batas Jatuh Tempo</th>
                                    <th>Tgl Dikembalikan</th>
                                    <th>Status</th>
                                    <th>Denda</th>
                                    <th style="text-align: right;">Aksi</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${data.data.map(t => `
                                    <tr>
                                        <td><code>${t.transaction_code}</code></td>
                                        <td><strong>${t.book_title || '-'}</strong></td>
                                        <td>${t.borrow_date}</td>
                                        <td><strong>${t.due_date}</strong></td>
                                        <td>${t.return_date || '-'}</td>
                                        <td>
                                            <span class="badge badge-${t.status}">
                                                ${t.status === 'borrowed' ? 'Sedang Dipinjam' : (t.status === 'returned' ? 'Sudah Kembali' : 'Terlambat')}
                                            </span>
                                        </td>
                                        <td>
                                            ${t.fine_amount > 0 ? `<strong style="color: var(--danger);">Rp ${Number(t.fine_amount).toLocaleString('id-ID')}</strong>` : '-'}
                                        </td>
                                        <td style="text-align: right;">
                                            ${t.status !== 'returned' ? `
                                                <button class="btn btn-sm btn-success" onclick="app.showReturnModal(${t.id}, '${t.book_title}', ${t.fine_amount})">
                                                    <i class="fa-solid fa-rotate-left"></i> Kembalikan Buku
                                                </button>
                                            ` : '<span style="color: var(--success); font-weight: 600;"><i class="fa-solid fa-check-double"></i> Selesai</span>'}
                                        </td>
                                    </tr>
                                `).join('')}
                            </tbody>
                        </table>
                    </div>
                `;
            }
        } catch (e) {
            console.error("Load my borrows error:", e);
        }
    },

    // ==========================================
    // MODALS & ACTIONS
    // ==========================================
    async loadCategories() {
        try {
            const res = await fetch(`${API_BASE}/categories`);
            const data = await res.json();
            if (data.success) {
                this.state.categories = data.data;
            }
        } catch (e) {
            console.error("Load categories error:", e);
        }
    },

    openModal(htmlContent) {
        const box = document.getElementById('modal-box');
        const backdrop = document.getElementById('modal-backdrop');
        box.innerHTML = htmlContent;
        backdrop.classList.add('active');
    },

    closeAllModals() {
        const backdrop = document.getElementById('modal-backdrop');
        if (backdrop) backdrop.classList.remove('active');
    },

    showLoginModal(prefillUser = '') {
        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-user-lock"></i> Masuk ke Sistem</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.login(this.username.value, this.password.value)">
                <div class="modal-body">
                    <div class="form-group">
                        <label class="form-label">Username atau NIS / NIP</label>
                        <input type="text" name="username" class="form-control" placeholder="Contoh: admin atau 202510001" value="${prefillUser}" required autofocus>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Kata Sandi (Password)</label>
                        <input type="password" name="password" class="form-control" placeholder="Masukkan kata sandi..." required>
                    </div>
                    <div style="background: #f8fafc; padding: 0.85rem; border-radius: var(--radius-md); font-size: 0.8rem; color: var(--text-muted); margin-bottom: 1rem; border: 1px solid var(--border-color);">
                        <i class="fa-solid fa-circle-info" style="color: var(--primary);"></i> <strong>Akun Pengujian Demo:</strong><br>
                        &bull; Admin: <code>admin</code> / <code>admin123</code><br>
                        &bull; Siswa: <code>siswa1</code> / <code>siswa123</code>
                    </div>
                </div>
                <div class="modal-footer" style="justify-content: space-between;">
                    <button type="button" class="btn btn-outline" onclick="app.showRegisterModal()">
                        <i class="fa-solid fa-user-plus"></i> Daftar Siswa
                    </button>
                    <button type="submit" class="btn btn-primary">
                        <i class="fa-solid fa-right-to-bracket"></i> Masuk Sekarang
                    </button>
                </div>
            </form>
        `);
    },

    showRegisterModal() {
        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-user-plus"></i> Pendaftaran Akun Siswa</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.handleRegisterForm(this)">
                <div class="modal-body">
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">NIS (Nomor Induk Siswa)</label>
                            <input type="text" name="nis_nip" class="form-control" placeholder="Contoh: 202510003" required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Kelas</label>
                            <input type="text" name="class_name" class="form-control" placeholder="Contoh: XII RPL 1" required>
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Nama Lengkap</label>
                        <input type="text" name="full_name" class="form-control" placeholder="Masukkan nama lengkap siswa..." required>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Username</label>
                            <input type="text" name="username" class="form-control" placeholder="Username untuk login" required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Password</label>
                            <input type="password" name="password" class="form-control" placeholder="Minimal 6 karakter" required>
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Email (Opsional)</label>
                            <input type="email" name="email" class="form-control" placeholder="siswa@email.com">
                        </div>
                        <div class="form-group">
                            <label class="form-label">No. Telepon / WA (Opsional)</label>
                            <input type="text" name="phone" class="form-control" placeholder="08xxxxxxxxxx">
                        </div>
                    </div>
                </div>
                <div class="modal-footer" style="justify-content: space-between;">
                    <button type="button" class="btn btn-outline" onclick="app.showLoginModal()">
                        Sudah punya akun? Masuk
                    </button>
                    <button type="submit" class="btn btn-primary">
                        <i class="fa-solid fa-check"></i> Daftarkan Akun
                    </button>
                </div>
            </form>
        `);
    },

    handleRegisterForm(form) {
        const formData = {
            nis_nip: form.nis_nip.value,
            class_name: form.class_name.value,
            full_name: form.full_name.value,
            username: form.username.value,
            password: form.password.value,
            email: form.email.value || null,
            phone: form.phone.value || null,
            role: 'siswa'
        };
        this.register(formData);
    },

    async showBorrowModal(bookId) {
        if (!this.state.user) {
            this.showLoginModal();
            return;
        }

        const book = this.state.books.find(b => b.id === bookId);
        if (!book) return;

        let memberSelectHtml = '';
        if (this.state.user.role === 'admin') {
            // Admin can borrow on behalf of students
            const res = await fetch(`${API_BASE}/members`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const members = data.data || [];
            memberSelectHtml = `
                <div class="form-group">
                    <label class="form-label">Pilih Siswa Peminjam</label>
                    <select name="user_id" class="form-control" required>
                        ${members.filter(m => m.role === 'siswa').map(m => `<option value="${m.id}">${m.full_name} (${m.nis_nip} - ${m.class_name || 'Siswa'})</option>`).join('')}
                    </select>
                </div>
            `;
        }

        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-hand-holding-hand"></i> Konfirmasi Peminjaman Buku</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.processBorrow(${bookId}, this)">
                <div class="modal-body">
                    <div style="display: flex; gap: 1rem; background: #f8fafc; padding: 1rem; border-radius: var(--radius-md); margin-bottom: 1.25rem; border: 1px solid var(--border-color);">
                        <div style="flex: 1;">
                            <h4 style="color: var(--primary); font-weight: 800; font-size: 1.05rem;">${book.title}</h4>
                            <p style="font-size: 0.85rem; color: var(--text-muted); margin-top: 0.25rem;">Pengarang: <strong>${book.author}</strong> &bull; ISBN: ${book.isbn}</p>
                            <p style="font-size: 0.8rem; color: var(--text-muted); margin-top: 0.25rem;">Stok Tersedia: <strong style="color: var(--success);">${book.stock}</strong> buku</p>
                        </div>
                    </div>

                    ${memberSelectHtml}

                    <div class="form-group">
                        <label class="form-label">Durasi Peminjaman (Hari)</label>
                        <select name="borrow_duration_days" class="form-control">
                            <option value="3">3 Hari</option>
                            <option value="7" selected>7 Hari (Standar 1 Minggu)</option>
                            <option value="14">14 Hari (2 Minggu)</option>
                        </select>
                    </div>

                    <div class="form-group">
                        <label class="form-label">Catatan Peminjaman (Opsional)</label>
                        <input type="text" name="notes" class="form-control" placeholder="Misal: Keperluan tugas praktik kelompok">
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                    <button type="submit" class="btn btn-primary">
                        <i class="fa-solid fa-check"></i> Konfirmasi Pinjam
                    </button>
                </div>
            </form>
        `);
    },

    async processBorrow(bookId, form) {
        const payload = {
            book_id: bookId,
            borrow_duration_days: parseInt(form.borrow_duration_days.value, 10),
            notes: form.notes.value || null
        };
        if (form.user_id) {
            payload.user_id = parseInt(form.user_id.value, 10);
        }

        try {
            const res = await fetch(`${API_BASE}/transactions/borrow`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.state.token}`
                },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                this.closeAllModals();
                this.showToast(data.message, 'success');
                await this.loadBooks();
                if (this.state.currentView === 'my_borrows') this.navigate('my_borrows');
                if (this.state.currentView === 'dashboard') this.navigate('dashboard');
                if (this.state.currentView === 'transactions_admin') this.navigate('transactions_admin');
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal memproses peminjaman', 'error');
        }
    },

    showReturnModal(trxId, bookTitle, fineAmount) {
        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-rotate-left"></i> Pengembalian Buku</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.processReturn(${trxId}, this.notes.value)">
                <div class="modal-body">
                    <p style="font-size: 0.95rem; color: var(--text-main); margin-bottom: 1rem;">
                        Apakah Anda yakin ingin mengembalikan buku <strong>"${bookTitle}"</strong> ke perpustakaan?
                    </p>

                    ${fineAmount > 0 ? `
                        <div style="background: var(--danger-bg); border: 1px solid var(--danger-border); padding: 1rem; border-radius: var(--radius-md); margin-bottom: 1rem;">
                            <h4 style="color: var(--danger); font-size: 0.9rem; font-weight: 700; margin-bottom: 0.25rem;">
                                <i class="fa-solid fa-triangle-exclamation"></i> Peringatan Denda Keterlambatan!
                            </h4>
                            <p style="font-size: 0.85rem; color: var(--text-main);">
                                Buku ini melewati batas tanggal jatuh tempo. Total denda yang harus dibayarkan: <strong style="color: var(--danger); font-size: 1.1rem;">Rp ${Number(fineAmount).toLocaleString('id-ID')}</strong> (Rp 1.000/hari).
                            </p>
                        </div>
                    ` : `
                        <div style="background: var(--success-bg); border: 1px solid var(--success-border); padding: 0.85rem; border-radius: var(--radius-md); margin-bottom: 1rem; font-size: 0.85rem; color: var(--success);">
                            <i class="fa-solid fa-circle-check"></i> Pengembalian tepat waktu. Tidak dikenakan denda.
                        </div>
                    `}

                    <div class="form-group">
                        <label class="form-label">Catatan Pengembalian (Opsional)</label>
                        <input type="text" name="notes" class="form-control" placeholder="Contoh: Kondisi buku sangat baik">
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                    <button type="submit" class="btn btn-success">
                        <i class="fa-solid fa-check"></i> Selesaikan Pengembalian
                    </button>
                </div>
            </form>
        `);
    },

    async processReturn(trxId, notes) {
        try {
            const res = await fetch(`${API_BASE}/transactions/return`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.state.token}`
                },
                body: JSON.stringify({ transaction_id: trxId, notes: notes || null })
            });
            const data = await res.json();
            if (data.success) {
                this.closeAllModals();
                this.showToast(data.message, 'success');
                if (this.state.currentView === 'my_borrows') this.navigate('my_borrows');
                if (this.state.currentView === 'dashboard') this.navigate('dashboard');
                if (this.state.currentView === 'transactions_admin') this.navigate('transactions_admin');
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal memproses pengembalian', 'error');
        }
    },

    // CRUD BUKU (ADMIN)
    showAddBookModal() {
        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-plus-circle"></i> Tambah Buku Baru</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.saveBook(null, this)">
                <div class="modal-body">
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Nomor ISBN</label>
                            <input type="text" name="isbn" class="form-control" placeholder="978-602-..." required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Kategori</label>
                            <select name="category_id" class="form-control" required>
                                ${this.state.categories.map(c => `<option value="${c.id}">${c.name}</option>`).join('')}
                            </select>
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Judul Buku</label>
                        <input type="text" name="title" class="form-control" placeholder="Masukkan judul buku..." required>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Pengarang / Penulis</label>
                            <input type="text" name="author" class="form-control" placeholder="Nama pengarang" required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Penerbit</label>
                            <input type="text" name="publisher" class="form-control" placeholder="Nama penerbit" required>
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Tahun Terbit</label>
                            <input type="number" name="year" class="form-control" value="2024" required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Jumlah Stok Eksemplar</label>
                            <input type="number" name="stock" class="form-control" value="10" min="1" required>
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Lokasi Rak (Opsional)</label>
                            <input type="text" name="shelf_location" class="form-control" placeholder="Misal: Rak A-01">
                        </div>
                        <div class="form-group">
                            <label class="form-label">URL Gambar Sampul (Opsional)</label>
                            <input type="text" name="cover_url" class="form-control" placeholder="https://...">
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Sinopsis / Deskripsi Singkat</label>
                        <textarea name="description" class="form-control" placeholder="Tuliskan ringkasan buku..."></textarea>
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                    <button type="submit" class="btn btn-primary">
                        <i class="fa-solid fa-floppy-disk"></i> Simpan Buku
                    </button>
                </div>
            </form>
        `);
    },

    async showEditBookModal(bookId) {
        try {
            const res = await fetch(`${API_BASE}/books/${bookId}`);
            const data = await res.json();
            if (!data.success) return;
            const b = data.data;

            this.openModal(`
                <div class="modal-header">
                    <h3><i class="fa-solid fa-pen-to-square"></i> Edit Data Buku</h3>
                    <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
                </div>
                <form onsubmit="event.preventDefault(); app.saveBook(${b.id}, this)">
                    <div class="modal-body">
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">Nomor ISBN</label>
                                <input type="text" name="isbn" class="form-control" value="${b.isbn}" required>
                            </div>
                            <div class="form-group">
                                <label class="form-label">Kategori</label>
                                <select name="category_id" class="form-control" required>
                                    ${this.state.categories.map(c => `<option value="${c.id}" ${b.category_id === c.id ? 'selected' : ''}>${c.name}</option>`).join('')}
                                </select>
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Judul Buku</label>
                            <input type="text" name="title" class="form-control" value="${b.title}" required>
                        </div>
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">Pengarang / Penulis</label>
                                <input type="text" name="author" class="form-control" value="${b.author}" required>
                            </div>
                            <div class="form-group">
                                <label class="form-label">Penerbit</label>
                                <input type="text" name="publisher" class="form-control" value="${b.publisher}" required>
                            </div>
                        </div>
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">Tahun Terbit</label>
                                <input type="number" name="year" class="form-control" value="${b.year}" required>
                            </div>
                            <div class="form-group">
                                <label class="form-label">Stok Tersedia Saat Ini</label>
                                <input type="number" name="stock" class="form-control" value="${b.stock}" required>
                            </div>
                        </div>
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">Total Stok Aset</label>
                                <input type="number" name="total_stock" class="form-control" value="${b.total_stock}" required>
                            </div>
                            <div class="form-group">
                                <label class="form-label">Lokasi Rak</label>
                                <input type="text" name="shelf_location" class="form-control" value="${b.shelf_location || ''}">
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">URL Gambar Sampul</label>
                            <input type="text" name="cover_url" class="form-control" value="${b.cover_url || ''}">
                        </div>
                        <div class="form-group">
                            <label class="form-label">Deskripsi</label>
                            <textarea name="description" class="form-control">${b.description || ''}</textarea>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                        <button type="submit" class="btn btn-primary">
                            <i class="fa-solid fa-floppy-disk"></i> Perbarui Buku
                        </button>
                    </div>
                </form>
            `);
        } catch (e) {
            this.showToast('Gagal memuat data buku', 'error');
        }
    },

    async saveBook(bookId, form) {
        const payload = {
            isbn: form.isbn.value,
            title: form.title.value,
            author: form.author.value,
            publisher: form.publisher.value,
            year: parseInt(form.year.value, 10),
            category_id: parseInt(form.category_id.value, 10),
            stock: parseInt(form.stock.value, 10),
            total_stock: form.total_stock ? parseInt(form.total_stock.value, 10) : parseInt(form.stock.value, 10),
            shelf_location: form.shelf_location ? form.shelf_location.value || null : null,
            cover_url: form.cover_url ? form.cover_url.value || null : null,
            description: form.description ? form.description.value || null : null,
        };

        const isEdit = bookId !== null;
        const url = isEdit ? `${API_BASE}/books/${bookId}` : `${API_BASE}/books`;
        const method = isEdit ? 'PUT' : 'POST';

        try {
            const res = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.state.token}`
                },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                this.closeAllModals();
                this.showToast(data.message, 'success');
                if (this.state.currentView === 'books_admin') {
                    this.loadAdminBooksTable();
                } else {
                    this.loadBooks();
                }
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal menyimpan buku', 'error');
        }
    },

    async deleteBook(bookId, bookTitle) {
        if (!confirm(`Hapus data buku "${bookTitle}"? Tindakan ini tidak dapat dibatalkan.`)) return;
        try {
            const res = await fetch(`${API_BASE}/books/${bookId}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            if (data.success) {
                this.showToast(data.message, 'success');
                this.loadAdminBooksTable();
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal menghapus buku', 'error');
        }
    },

    // CRUD ANGGOTA (ADMIN)
    showAddMemberModal() {
        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-user-plus"></i> Tambah Anggota Siswa Baru</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.saveMember(null, this)">
                <div class="modal-body">
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">NIS (Nomor Induk Siswa)</label>
                            <input type="text" name="nis_nip" class="form-control" placeholder="Contoh: 202510005" required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Kelas</label>
                            <input type="text" name="class_name" class="form-control" placeholder="Contoh: XII RPL 2" required>
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Nama Lengkap</label>
                        <input type="text" name="full_name" class="form-control" placeholder="Nama lengkap siswa..." required>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Username</label>
                            <input type="text" name="username" class="form-control" placeholder="Username login" required>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Password Awal</label>
                            <input type="password" name="password" class="form-control" placeholder="Password siswa" required>
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">Email</label>
                            <input type="email" name="email" class="form-control" placeholder="siswa@email.com">
                        </div>
                        <div class="form-group">
                            <label class="form-label">No. Telepon / WA</label>
                            <input type="text" name="phone" class="form-control" placeholder="08xxxxxxxxxx">
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                    <button type="submit" class="btn btn-primary">
                        <i class="fa-solid fa-check"></i> Simpan Anggota
                    </button>
                </div>
            </form>
        `);
    },

    async showEditMemberModal(memberId) {
        try {
            const res = await fetch(`${API_BASE}/members`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const member = (data.data || []).find(m => m.id === memberId);
            if (!member) return;

            this.openModal(`
                <div class="modal-header">
                    <h3><i class="fa-solid fa-user-pen"></i> Edit Data Anggota</h3>
                    <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
                </div>
                <form onsubmit="event.preventDefault(); app.saveMember(${member.id}, this)">
                    <div class="modal-body">
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">NIS / NIP</label>
                                <input type="text" name="nis_nip" class="form-control" value="${member.nis_nip}" required>
                            </div>
                            <div class="form-group">
                                <label class="form-label">Kelas</label>
                                <input type="text" name="class_name" class="form-control" value="${member.class_name || ''}">
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Nama Lengkap</label>
                            <input type="text" name="full_name" class="form-control" value="${member.full_name}" required>
                        </div>
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">Email</label>
                                <input type="email" name="email" class="form-control" value="${member.email || ''}">
                            </div>
                            <div class="form-group">
                                <label class="form-label">No. Telepon / WA</label>
                                <input type="text" name="phone" class="form-control" value="${member.phone || ''}">
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Reset Password (Kosongkan jika tidak diganti)</label>
                            <input type="password" name="password" class="form-control" placeholder="Masukkan password baru...">
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                        <button type="submit" class="btn btn-primary">
                            <i class="fa-solid fa-floppy-disk"></i> Perbarui Anggota
                        </button>
                    </div>
                </form>
            `);
        } catch (e) {
            this.showToast('Gagal memuat anggota', 'error');
        }
    },

    async saveMember(memberId, form) {
        const isEdit = memberId !== null;
        const payload = {
            nis_nip: form.nis_nip.value,
            full_name: form.full_name.value,
            class_name: form.class_name ? form.class_name.value || null : null,
            email: form.email ? form.email.value || null : null,
            phone: form.phone ? form.phone.value || null : null,
            password: form.password ? form.password.value || null : null,
        };

        if (!isEdit) {
            payload.username = form.username.value;
            payload.role = 'siswa';
        }

        const url = isEdit ? `${API_BASE}/members/${memberId}` : `${API_BASE}/members`;
        const method = isEdit ? 'PUT' : 'POST';

        try {
            const res = await fetch(url, {
                method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.state.token}`
                },
                body: JSON.stringify(payload)
            });
            const data = await res.json();
            if (data.success) {
                this.closeAllModals();
                this.showToast(data.message, 'success');
                this.loadAdminMembersTable();
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal menyimpan anggota', 'error');
        }
    },

    async deleteMember(memberId, memberName) {
        if (!confirm(`Hapus anggota "${memberName}"?`)) return;
        try {
            const res = await fetch(`${API_BASE}/members/${memberId}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            if (data.success) {
                this.showToast(data.message, 'success');
                this.loadAdminMembersTable();
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal menghapus anggota', 'error');
        }
    },

    async deleteTransaction(trxId) {
        if (!confirm('Hapus catatan transaksi ini dari riwayat?')) return;
        try {
            const res = await fetch(`${API_BASE}/transactions/${trxId}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            if (data.success) {
                this.showToast(data.message, 'success');
                this.loadAdminTransactionsTable();
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal menghapus transaksi', 'error');
        }
    },

    showAddCategoryModal() {
        this.openModal(`
            <div class="modal-header">
                <h3><i class="fa-solid fa-tags"></i> Tambah Kategori Buku</h3>
                <button class="modal-close-btn" onclick="app.closeAllModals()">&times;</button>
            </div>
            <form onsubmit="event.preventDefault(); app.saveCategory(this.name.value, this.description.value)">
                <div class="modal-body">
                    <div class="form-group">
                        <label class="form-label">Nama Kategori</label>
                        <input type="text" name="name" class="form-control" placeholder="Contoh: Desain Grafis, Robotika" required>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Deskripsi Kategori</label>
                        <textarea name="description" class="form-control" placeholder="Penjelasan kategori..."></textarea>
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-outline" onclick="app.closeAllModals()">Batal</button>
                    <button type="submit" class="btn btn-primary">
                        <i class="fa-solid fa-check"></i> Simpan Kategori
                    </button>
                </div>
            </form>
        `);
    },

    async saveCategory(name, description) {
        try {
            const res = await fetch(`${API_BASE}/categories`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.state.token}`
                },
                body: JSON.stringify({ name, description: description || null })
            });
            const data = await res.json();
            if (data.success) {
                this.closeAllModals();
                this.showToast(data.message, 'success');
                await this.loadCategories();
            } else {
                this.showToast(data.message, 'error');
            }
        } catch (e) {
            this.showToast('Gagal menyimpan kategori', 'error');
        }
    },

    // ==========================================
    // CETAK LAPORAN (PRINT LAYOUT)
    // ==========================================
    async printReport() {
        try {
            const res = await fetch(`${API_BASE}/transactions`, {
                headers: { 'Authorization': `Bearer ${this.state.token}` }
            });
            const data = await res.json();
            const list = data.data || [];

            const printContainer = document.getElementById('print-report-container');
            const todayStr = new Date().toLocaleDateString('id-ID', { day: 'numeric', month: 'long', year: 'numeric' });

            printContainer.innerHTML = `
                <div style="font-family: Arial, sans-serif; padding: 20px; line-height: 1.4;">
                    <!-- Header Kop Surat Sekolah -->
                    <div style="text-align: center; border-bottom: 2px solid #000; padding-bottom: 12px; margin-bottom: 20px;">
                        <h2 style="margin: 0; font-size: 1.4rem; text-transform: uppercase;">PEMERINTAH PROVINSI / DAERAH</h2>
                        <h1 style="margin: 4px 0; font-size: 1.7rem; text-transform: uppercase;">SMK REKAYASA PERANGKAT LUNAK</h1>
                        <h3 style="margin: 0; font-size: 1.1rem; font-weight: normal;">UNIT PELAKSANA TEKNIS PERPUSTAKAAN DIGITAL</h3>
                        <p style="margin: 4px 0 0 0; font-size: 0.85rem;">Jl. Pendidikan Kejuruan No. 4, Telp. (021) 12345678, Web: perpustakaan.sch.id</p>
                    </div>

                    <div style="text-align: center; margin-bottom: 20px;">
                        <h3 style="margin: 0; text-decoration: underline; text-transform: uppercase;">LAPORAN REKAPITULASI TRANSAKSI PEMINJAMAN BUKU</h3>
                        <p style="margin: 4px 0 0 0; font-size: 0.9rem;">Dicetak pada tanggal: <strong>${todayStr}</strong></p>
                    </div>

                    <table style="width: 100%; border-collapse: collapse; font-size: 0.85rem; margin-bottom: 30px;">
                        <thead>
                            <tr style="background: #e2e8f0;">
                                <th style="border: 1px solid #000; padding: 6px;">No</th>
                                <th style="border: 1px solid #000; padding: 6px;">Kode TRX</th>
                                <th style="border: 1px solid #000; padding: 6px;">Nama Siswa (NIS)</th>
                                <th style="border: 1px solid #000; padding: 6px;">Judul Buku</th>
                                <th style="border: 1px solid #000; padding: 6px;">Tgl Pinjam</th>
                                <th style="border: 1px solid #000; padding: 6px;">Jatuh Tempo</th>
                                <th style="border: 1px solid #000; padding: 6px;">Tgl Kembali</th>
                                <th style="border: 1px solid #000; padding: 6px;">Status</th>
                                <th style="border: 1px solid #000; padding: 6px;">Denda</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${list.map((t, idx) => `
                                <tr>
                                    <td style="border: 1px solid #000; padding: 6px; text-align: center;">${idx + 1}</td>
                                    <td style="border: 1px solid #000; padding: 6px;">${t.transaction_code}</td>
                                    <td style="border: 1px solid #000; padding: 6px;">${t.user_name} (${t.nis_nip})</td>
                                    <td style="border: 1px solid #000; padding: 6px;">${t.book_title}</td>
                                    <td style="border: 1px solid #000; padding: 6px; text-align: center;">${t.borrow_date}</td>
                                    <td style="border: 1px solid #000; padding: 6px; text-align: center;">${t.due_date}</td>
                                    <td style="border: 1px solid #000; padding: 6px; text-align: center;">${t.return_date || '-'}</td>
                                    <td style="border: 1px solid #000; padding: 6px; text-align: center;">
                                        ${t.status === 'borrowed' ? 'Dipinjam' : (t.status === 'returned' ? 'Kembali' : 'Terlambat')}
                                    </td>
                                    <td style="border: 1px solid #000; padding: 6px; text-align: right;">
                                        ${t.fine_amount > 0 ? `Rp ${Number(t.fine_amount).toLocaleString('id-ID')}` : '-'}
                                    </td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>

                    <!-- Tanda Tangan Penguji & Petugas -->
                    <div style="display: flex; justify-content: space-between; margin-top: 40px; font-size: 0.9rem;">
                        <div style="text-align: center; width: 220px;">
                            <p>Mengetahui,<br><strong>Penguji UKK RPL</strong></p>
                            <div style="height: 60px;"></div>
                            <p>_______________________<br>NIP/NUPTK.</p>
                        </div>
                        <div style="text-align: center; width: 220px;">
                            <p>Kota, ${todayStr}<br><strong>Petugas Perpustakaan</strong></p>
                            <div style="height: 60px;"></div>
                            <p><strong>${this.state.user ? this.state.user.full_name : 'Administrator'}</strong><br>NIP. 198501012010011001</p>
                        </div>
                    </div>
                </div>
            `;

            window.print();
        } catch (e) {
            this.showToast('Gagal menyiapkan laporan cetak', 'error');
        }
    },

    // ==========================================
    // TOAST NOTIFICATIONS
    // ==========================================
    showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        if (!container) return;

        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        
        const icon = type === 'success' ? 'fa-circle-check' : (type === 'error' ? 'fa-circle-xmark' : 'fa-circle-info');
        toast.innerHTML = `
            <i class="fa-solid ${icon} toast-icon"></i>
            <span class="toast-text">${message}</span>
        `;

        container.appendChild(toast);
        setTimeout(() => {
            toast.style.opacity = '0';
            toast.style.transform = 'translateX(100%)';
            toast.style.transition = 'all 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, 3500);
    }
};

// Initialize App on DOMContentLoaded
document.addEventListener('DOMContentLoaded', () => {
    app.init();
});
