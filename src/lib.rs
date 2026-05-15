pub mod scaffolding;
pub mod database;
pub mod monitoring;
pub mod builder;
pub mod utils;
pub mod auth;

pub use scaffolding::*;
pub use database::*;
pub use monitoring::*;
pub use builder::*;
pub use utils::*;
pub use auth::*;

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
    println!("\n{}", "🛠️  RustBasic CLI".magenta().bold());
    println!("{}", "=================".magenta());
    println!("Gunakan 'rustbasic --help' untuk daftar perintah lengkap.");
}
