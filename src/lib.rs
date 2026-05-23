pub mod scaffolding;
pub mod database;
pub mod monitoring;
pub mod builder;
pub mod utils;
pub use scaffolding::*;
pub use database::*;
pub use monitoring::*;
pub use builder::*;
pub use utils::*;

use std::env;
use dotenvy::dotenv;
use colored::*;
use std::future::Future;
use std::pin::Pin;

pub type AsyncHook = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>;

pub async fn run_cli<F, G>(migrate_fn: F, seed_fn: G) 
where 
    F: Fn(String) -> Pin<Box<dyn Future<Output = Result<(), String>>>>,
    G: Fn() -> Pin<Box<dyn Future<Output = ()>>>
{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = &args[1];

    // .env hanya diwajibkan untuk perintah selain 'new'
    if command != "new" {
        let _ = dotenv();
        ensure_session().await;
    }

    match command.as_str() {
        "migrate" | "migrate:refresh" | "migrate:back" | "migrate:rollback" => {
             match migrate_fn(command.clone()).await {
                Ok(_) => println!("\n{} Operasi '{}' berhasil.", "✅".green(), command),
                Err(e) => eprintln!("\n{} Gagal: {}", "❌".red(), e),
             }
        }
        "db:seed" => {
            println!("{}", "🌱 Menjalankan seeder database...".cyan());
            seed_fn().await;
            println!("\n{} Database seeding berhasil.", "✅".green());
        }
        _ => {
            // Perintah lainnya ditangani oleh main.rs global atau binari ini jika dipanggil langsung
            // Tapi biasanya binari lokal hanya dipanggil untuk migrate/seed
        }
    }
}

pub fn print_help() {
    println!("Gunakan 'rustbasic' untuk melihat opsi perintah.");
}

/// Fungsi utama untuk menangani CLI di tingkat project (dipanggil oleh project main.rs)
pub async fn handle<M: sea_orm_migration::MigratorTrait>(args: &[String]) -> bool {
    if args.len() < 2 {
        return false;
    }

    let command = args[1].as_str();
    
    // Daftar perintah yang ditangani oleh CLI lokal project
    let is_migration_cmd = command.starts_with("migrate") || command == "db:seed";
    let is_storage_cmd = command == "storage:link";
    
    if !is_migration_cmd && !is_storage_cmd {
        return false;
    }

    ensure_session().await;

    println!("{} {}", "🛠️  RustBasic Local CLI - Command:".magenta().bold(), command.yellow());

    if is_storage_cmd {
        handle_storage_link();
        return true;
    }

    // Gunakan fungsi connect lokal
    let db = crate::database::connect().await;

    match command {
        "migrate" => {
            println!("🚀 {}", "Menjalankan migrasi database...".cyan());
            if let Err(e) = M::up(&db, None).await {
                println!("❌ {} {}", "Gagal menjalankan migrasi:".red().bold(), e);
            } else {
                println!("✅ {}", "Migrasi selesai!".green().bold());
            }
        }
        "migrate:refresh" => {
            println!("🔄 {}", "Mereset dan menjalankan ulang migrasi...".cyan());
            if let Err(e) = M::fresh(&db).await {
                println!("❌ {} {}", "Gagal refresh migrasi:".red().bold(), e);
            } else {
                println!("✅ {}", "Database berhasil di-refresh!".green().bold());
            }
        }
        "migrate:back" | "migrate:rollback" => {
            println!("⬅️  {}", "Rollback migrasi terakhir...".cyan());
            if let Err(e) = M::down(&db, None).await {
                println!("❌ {} {}", "Gagal rollback:".red().bold(), e);
            } else {
                println!("✅ {}", "Rollback berhasil!".green().bold());
            }
        }
        "db:seed" => {
            println!("🌱 {}", "Fitur db:seed membutuhkan implementasi lokal.".yellow());
        }
        _ => return false,
    }

    true
}

/// Membuat symbolic link dari public/storage ke storage/app/public
fn handle_storage_link() {
    let target = "public/storage";
    let source = "storage/app/public";

    // 1. Buat folder source jika belum ada
    if let Err(e) = std::fs::create_dir_all(source) {
        println!("❌ {} {}", "Gagal membuat direktori storage:".red().bold(), e);
        return;
    }

    // 2. Cek apakah link sudah ada
    let path = std::path::Path::new(target);
    if path.exists() || path.is_symlink() {
        println!("ℹ️  {}", "Link 'public/storage' sudah ada atau berupa file/folder lain.".yellow());
        return;
    }

    // 3. Buat symlink
    println!("🔗 {}", "Membuat symbolic link...".cyan());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        // Gunakan path relatif agar tetap valid jika project dipindah
        if let Err(e) = symlink("../storage/app/public", target) {
            println!("❌ {} {}", "Gagal membuat symlink:".red().bold(), e);
        } else {
            println!("✅ {} [public/storage -> storage/app/public]", "Link storage berhasil dibuat!".green().bold());
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        if let Err(e) = symlink_dir("../storage/app/public", target) {
            println!("❌ {} {}", "Gagal membuat symlink:".red().bold(), e);
        } else {
            println!("✅ {} [public/storage -> storage/app/public]", "Link storage berhasil dibuat!".green().bold());
        }
    }
}
