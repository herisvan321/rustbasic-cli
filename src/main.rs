use std::env;
use dotenvy::dotenv;
use colored::*;
use rustbasic_cli::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str();

    // Perintah yang TIDAK butuh runtime async (Sangat Cepat)
    match command {
        "-v" | "--version" | "version" => {
            println!("{} {}", "🛠️  RustBasic CLI Version:".magenta().bold(), env!("CARGO_PKG_VERSION").cyan().bold());
            return;
        }
        "make:model" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama model tidak ditentukan.".red().bold());
                return;
            }
            let model_name = &args[2];
            let with_migration = args.contains(&"-m".to_string());
            scaffolding::make_model(model_name);
            if with_migration {
                scaffolding::make_rust_migration(model_name);
            }
            return;
        }
        "make:migration" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama migration tidak ditentukan.".red().bold());
                return;
            }
            scaffolding::make_rust_migration(&args[2]);
            return;
        }
        "make:migration:add" => {
            if args.len() < 4 {
                println!("{}", "❌ Error: Gunakan: rustbasic make:migration:add <kolom> <tabel>".red().bold());
                return;
            }
            scaffolding::make_rust_migration_add(&args[2], &args[3]);
            return;
        }
        "make:controller" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama controller tidak ditentukan.".red().bold());
                return;
            }
            scaffolding::make_controller(&args[2]);
            return;
        }
        "make:middleware" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama middleware tidak ditentukan.".red().bold());
                return;
            }
            scaffolding::make_middleware(&args[2]);
            return;
        }
        "make:seeder" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama seeder tidak ditentukan.".red().bold());
                return;
            }
            scaffolding::make_seeder(&args[2]);
            return;
        }
        "route:list" => {
            monitoring::list_routes();
            return;
        }
        "build" => {
            builder::build_project();
            println!("\n{} {}", "✅".green(), "Build project berhasil diselesaikan.".green().bold());
            return;
        }
        "check:update" => {
            monitoring::check_updates();
            println!("\n{} {}", "✅".green(), "Pemeriksaan update selesai.".green().bold());
            return;
        }
        "check:security" => {
            monitoring::check_security();
            println!("\n{} {}", "✅".green(), "Audit keamanan selesai.".green().bold());
            return;
        }
        "key:generate" => {
            database::generate_app_key();
            return;
        }
        "new" => {
            run_new_command(&args);
            return;
        }
        _ => {}
    }

    // Perintah yang BUTUH runtime async
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // .env hanya diwajibkan untuk perintah selain 'new'
        let _ = dotenv();

        match command {
            "serve" => {
                println!("\n   {} {}", "🚀".bold(), "Menjalankan server RustBasic dengan Auto-Reload...".magenta().bold());
                let host = env::var("APP_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("APP_PORT").unwrap_or_else(|_| "4000".to_string());
                let display_host = if host == "0.0.0.0" { "localhost".to_string() } else { host };
                let app_url = format!("http://{}:{}", display_host, port);
                utils::wait_and_open(app_url);

                let status = std::process::Command::new("cargo")
                    .args(["watch", "-c", "-q", "--no-ignore", "-i", "target", "-w", "src", "-w", ".env", "-w", "src/resources", "-x", "run"])
                    .status()
                    .expect("❌ Gagal menjalankan cargo watch.");
                
                if !status.success() {
                    std::process::exit(status.code().unwrap_or(1));
                }
            }
            "migrate" | "migrate:refresh" | "migrate:back" | "migrate:rollback" | "db:seed" | "storage:link" => {
                // Delegasi ke cargo run untuk memastikan kode lokal terbaca
                delegate_to_cargo(&args);
            }
            "cache:clear" => {
                database::clear_cache().await;
            }
            "make:auth" | "auth" => {
                if args.len() >= 3 && args[2] == "back" {
                    auth::remove_auth().await;
                    println!("\n{} {}", "✅".green(), "Scaffolding autentikasi berhasil dihapus.".green().bold());
                } else {
                    auth::make_auth().await;
                    println!("\n{} {}", "✅".green(), "Scaffolding autentikasi berhasil dibuat.".green().bold());
                }
            }
            "auth:back" => {
                auth::remove_auth().await;
            }
            _ => {
                println!("{} {}", "❌ Error: Perintah tidak dikenal:".red().bold(), command.yellow());
                print_help();
            }
        }
    });
}

fn delegate_to_cargo(args: &[String]) {
    if env::var("RUSTBASIC_LOCAL").is_err() && std::path::Path::new("Cargo.toml").exists() {
        let status = std::process::Command::new("cargo")
            .args(["run", "-q", "--"])
            .args(&args[1..])
            .env("RUSTBASIC_LOCAL", "true")
            .status();

        if let Ok(s) = status {
            std::process::exit(s.code().unwrap_or(0));
        }
    }
}

fn run_new_command(args: &[String]) {
    if args.len() < 3 {
        println!("{}", "❌ Error: Nama project tidak ditentukan.".red().bold());
        return;
    }
    let project_name = &args[2];
    if std::path::Path::new(project_name).exists() {
        println!("{} '{}' {}", "❌ Error: Folder".red().bold(), project_name.yellow(), "sudah ada!".red().bold());
        return;
    }

    println!("\n✨ {} {}", "Membuat project baru:".bold(), project_name.cyan().bold());
    let status = std::process::Command::new("git")
        .args(["clone", "https://github.com/herisvan321/rustbasic", project_name])
        .status();

    if let Ok(s) = status {
        if s.success() {
            let _ = std::process::Command::new("rm").args(["-rf", &format!("{}/.git", project_name)]).status();
            let env_example = format!("{}/.env.example", project_name);
            let env_file = format!("{}/.env", project_name);
            if std::path::Path::new(&env_example).exists() {
                let _ = std::fs::copy(&env_example, &env_file);
            }
            if std::env::set_current_dir(project_name).is_ok() {
                database::generate_app_key();
                println!("📦 {}", "Mengunduh dependencies...".bold());
                let _ = std::process::Command::new("cargo").args(["fetch"]).status();
                println!("\n✅ {}", "Project berhasil dibuat!".green().bold());
            }
        }
    }
}

fn print_help() {
    println!("\n{}", "🛠️  RustBasic CLI".magenta().bold());
    println!("{}", "=================".magenta());
    println!("{}", "Penggunaan:".bold());
    println!("  {} {} <Nama>         {}", "rustbasic".blue(), "new".green(), "Membuat project RustBasic baru".dimmed());
    println!("  {} {} <Nama>   {}", "rustbasic".blue(), "make:controller".green(), "Membuat controller baru".dimmed());
    println!("  {} {} <Nama> [-m]   {}", "rustbasic".blue(), "make:model".green(), "Membuat model & migration".dimmed());
    println!("  {} {} <Nama>    {}", "rustbasic".blue(), "make:migration".green(), "Membuat file migrasi (Create)".dimmed());
    println!("  {} {} <Kolom> <Tabel> {}", "rustbasic".blue(), "make:migration:add".green(), "Membuat file migrasi (Add Column)".dimmed());
    println!("  {} {} <Nama>     {}", "rustbasic".blue(), "make:middleware".green(), "Membuat middleware baru".dimmed());
    println!("  {} {} <Nama>       {}", "rustbasic".blue(), "make:seeder".green(), "Membuat seeder baru".dimmed());
    println!("  {} {}                  {}", "rustbasic".blue(), "migrate".green(), "Menjalankan migrasi database".dimmed());
    println!("  {} {}                {}", "rustbasic".blue(), "storage:link".green(), "Menghubungkan public/storage ke storage/app/public".dimmed());
    println!("  {} {}                   {}", "rustbasic".blue(), "serve".green(), "Menjalankan server (Auto-Reload)".dimmed());
    println!("  {} {}                 {}", "rustbasic".blue(), "version".green(), "Menampilkan versi CLI".dimmed());
    println!("\n💡 Gunakan 'rustbasic version' untuk informasi lebih lanjut.");
}
