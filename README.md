# 🛠️ RustBasic CLI

## 📝 Kata Pengantar

Selamat datang di **RustBasic CLI**. Perkakas baris perintah (Command-Line Interface) ini dirancang khusus untuk mempermudah dan mempercepat alur kerja pengembangan aplikasi menggunakan **RustBasic Framework**. Dengan memanfaatkan biner global yang memiliki waktu startup ultra-cepat (sekitar 0.008 detik), RustBasic CLI mengotomatisasi pembuatan berkas kode boilerplate, pengelolaan migrasi database relasional, symbolic links direktori penyimpanan, serta audit keamanan dan performa proyek Anda secara instan di terminal.

---

## 🛠️ Script Contoh

### A. Cara Menginstal CLI Secara Global
```bash
# Menginstal biner global menggunakan Cargo
cargo install rustbasic-cli

# Memverifikasi instalasi berhasil
rustbasic version
```

### B. Membuat Proyek SPA Baru & Menjalankan Server Pengembangan
```bash
# 1. Membuat direktori proyek baru dari template resmi
rustbasic new my-new-spa-project

# 2. Masuk ke folder proyek
cd my-new-spa-project

# 3. Jalankan server pengembangan lokal (Hot Reload otomatis)
rustbasic serve
```

### C. Menghasilkan Berkas Kontroller & Model Database Baru
```bash
# Membuat file controller baru di src/app/http/controllers/
rustbasic make:controller ArticleController

# Membuat file model beserta file migrasi database baru secara bersamaan (-m)
rustbasic make:model Product -m
```

### D. Pengelolaan Paket Tambahan (Package Manager)
```bash
# Menginstal paket autentikasi otomatis (Breeze)
rustbasic install rustbasic-breeze

# Menampilkan daftar seluruh paket resmi yang terinstal
rustbasic list packages

# Menghapus paket tambahan beserta berkas scaffolding-nya secara bersih
rustbasic uninstall rustbasic-breeze
```

---

## 🔄 Perbandingan Pemakaian (Pintasan CLI vs Operasi Manual)

Berikut adalah tabel perbandingan pemakaian antara menggunakan pintasan otomatis RustBasic CLI dan menulis kode secara manual:

| Kriteria Pekerjaan | Operasi Kode Manual | Menggunakan Pintasan RustBasic CLI |
| :--- | :--- | :--- |
| **Waktu Pembuatan File** | Lambat, harus membuat file baru & menyalin struktur dasar. | Instan, file boilerplate tergenerasi rapi dalam hitungan milidetik. |
| **Pendaftaran Migrasi** | Harus mendesain file migrasi & mendaftarkannya secara manual. | Otomatis dibuat dengan nama ter-timestamp yang siap diisi skema. |
| **Menampilkan Rute Web** | Harus membaca file routing satu per satu untuk audit URL. | Cukup jalankan perintah, seluruh URL dan middleware tercetak rapi. |
| **Kecepatan Startup CLI** | Lambat jika memicu async runtime penuh untuk utilitas kecil. | Sangat instan (~0.008s) karena mem-bypass runtime async runtime. |

---

## 📊 Tabel Ringkasan Daftar Perintah CLI Lengkap

Berikut adalah tabel perintah penting yang disediakan oleh RustBasic CLI:

| Perintah Terminal | Deskripsi Singkat Kegunaan Perintah | Output / Hasil Eksekusi |
| :--- | :--- | :--- |
| **`new <name>`** | Menginisiasi proyek baru dari template dasar. | Struktur folder proyek lengkap tergenerasi. |
| **`serve`** | Menjalankan server lokal RustBasic dengan watcher. | Server berjalan di port 4000 dengan Hot-Reload. |
| **`make:controller <Name>`**| Membuat template file kontroller baru. | File controller baru siap pakai di direktori controller. |
| **`make:model <Name> -m`** | Membuat model database beserta migrasinya. | Model di `src/app/models/` & berkas migrasi baru terbuat. |
| **`migrate`** | Menjalankan migrasi database yang tertunda. | Skema tabel database ter-update ke versi terbaru. |
| **`db:seed`** | Menjalankan pengisian data awal database (seeder). | Tabel database terisi data dummy awal otomatis. |
| **`route:list`** | Menampilkan seluruh daftar rute URL yang terdaftar. | Tabel rute & middleware tercetak di layar terminal. |
| **`key:generate`** | Menghasilkan kunci enkripsi aplikasi baru. | Kunci enkripsi acak tersimpan otomatis pada berkas `.env`. |
| **`install <package>`** | Menginstal paket resmi tambahan ke proyek. | Paket terdaftar di `Cargo.toml` & scaffolding selesai dijalankan. |
| **`list packages`** | Menampilkan daftar paket tambahan yang terinstal. | Seluruh nama, versi, dan deskripsi paket tercetak di layar. |
| **`uninstall <package>`**| Menghapus paket beserta seluruh berkas scaffolding-nya. | Paket dihapus secara bersih dari proyek. |

---

## 🏁 Penutup

Perkakas **RustBasic CLI** dirancang untuk menghilangkan kejenuhan menulis kode boilerplate secara manual, menyelaraskan konfigurasi database, dan mengoptimalkan siklus hidup pengembangan aplikasi Anda. Dengan menguasai perintah-perintah CLI ini, Anda dapat menghemat waktu berharga dan fokus sepenuhnya pada pembangunan logika bisnis utama aplikasi web Anda.
