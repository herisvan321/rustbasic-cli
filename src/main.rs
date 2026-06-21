use std::env;
use rustbasic_core::dotenvy::dotenv;
use rustbasic_core::colored::*;
use rustbasic_core::tokio;
use rustbasic_cli::*;

#[allow(clippy::collapsible_if)]
fn main() {
    let _ = dotenv();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str();

    // Jalankan pengecekan session di database sebelum menjalankan perintah apapun (jika .env ada)
    if command != "new" && std::path::Path::new(".env").exists() {
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async {
                database::ensure_session().await;
            });
        }
    }

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
        "make:test" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama test tidak ditentukan.".red().bold());
                return;
            }
            let is_unit = args.contains(&"--unit".to_string());
            scaffolding::make_test(&args[2], is_unit);
            return;
        }
        "make:observer" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama observer tidak ditentukan.".red().bold());
                return;
            }
            let model_arg = args.iter().find(|a| a.starts_with("--model="));
            let model_name = model_arg.map(|a| a.trim_start_matches("--model=").to_string());
            scaffolding::make_observer(&args[2], model_name.as_deref());
            return;
        }
        "make:service" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama service tidak ditentukan.".red().bold());
                return;
            }
            scaffolding::make_service(&args[2]);
            return;
        }
        "compile:run" => {
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("run");
            if args.len() > 2 {
                cmd.args(&args[2..]);
            }
            let status = utils::run_cargo_with_progress(cmd)
                .expect("❌ Gagal menjalankan cargo run.");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
            return;
        }
        "test" => {
            println!("\n{} {}", "🧪".bold(), "Menjalankan unit/integration testing RustBasic...".magenta().bold());
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("test");
            let status = utils::run_cargo_with_progress(cmd)
                .expect("❌ Gagal menjalankan cargo test.");
            
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
            return;
        }
        "route:list" => {
            monitoring::list_routes();
            return;
        }
        "build" => {
            delegate_to_cargo(&args);
            let run_android = args.iter().any(|arg| arg == "--android");
            let run_desktop = args.iter().any(|arg| arg == "--desktop");
            let run_docker = args.iter().any(|arg| arg == "--docker");
            
            if run_docker || run_android || run_desktop {
                // Gunakan build_interactive untuk Docker/Desktop/Android dengan flag eksplisit
                builder::build_interactive(&args);
            } else if args.len() > 2 {
                // Ada flag lain, delegasikan ke build_interactive
                builder::build_interactive(&args);
            } else {
                // Tanpa flag apapun, tampilkan menu interaktif
                builder::build_interactive(&args);
            }
            return;
        }
        "deploy" => {
            delegate_to_cargo(&args);
            builder::deploy_interactive();
            return;
        }
        "clean" => {
            println!("\n🧹 Membersihkan proyek RustBasic...");
            println!("📦 Menghapus hasil kompilasi Rust (cargo clean)...");
            let cargo_clean = std::process::Command::new("cargo")
                .arg("clean")
                .spawn();
            
            let mut cargo_success = false;
            if let Ok(mut child) = cargo_clean {
                if let Ok(status) = child.wait() {
                    cargo_success = status.success();
                }
            }
            
            if cargo_success {
                println!("✅ Folder target kompilasi lokal berhasil dihapus.");
            } else {
                println!("⚠️  Gagal menjalankan 'cargo clean' atau command tidak ditemukan.");
            }

            // Hapus sisa file tar ekspor docker jika ada
            let tar_files = vec!["rustbasic.tar", "rustbasic.tar.gz"];
            for file in tar_files {
                let path = std::path::Path::new(file);
                if path.exists() {
                    let _ = std::fs::remove_file(path);
                    println!("🗑️  Berhasil menghapus file ekspor: {}", file);
                }
            }
            
            // Hapus temporary .tmp-rustbasic.tar.* files
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(".tmp-rustbasic.tar") || name.starts_with("rustbasic.tar") {
                            let _ = std::fs::remove_file(entry.path());
                            println!("🗑️  Berhasil menghapus file temp: {}", name);
                        }
                    }
                }
            }

            // Docker cache prune
            println!("🐳 Mencoba membersihkan cache builder Docker...");
            let _ = std::process::Command::new("docker")
                .args(&["builder", "prune", "-f"])
                .status();
            
            println!("\n✨ Semua proses pembersihan selesai!");
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
        "install" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama package tidak ditentukan.".red().bold());
                println!("   Penggunaan: {} {} {}", "rustbasic".blue(), "install".green(), "<nama-package>".cyan());
                return;
            }
            rustbasic_cli::packages::install_package(&args[2]);
            return;
        }
        "uninstall" => {
            if args.len() < 3 {
                println!("{}", "❌ Error: Nama package tidak ditentukan.".red().bold());
                println!("   Penggunaan: {} {} {}", "rustbasic".blue(), "uninstall".green(), "<nama-package>".cyan());
                return;
            }
            rustbasic_cli::packages::uninstall_package(&args[2]);
            return;
        }
        "list" => {
            if args.len() >= 3 && args[2] == "packages" {
                rustbasic_cli::packages::list_packages();
            } else {
                println!("{}", "❌ Error: Subcommand tidak dikenal.".red().bold());
                println!("   Penggunaan: {} {} {}", "rustbasic".blue(), "list".green(), "packages".cyan());
            }
            return;
        }
        "publish" => {
            let target = args.get(2).map(|s| s.as_str()).unwrap_or("");
            rustbasic_cli::publisher::publish_config(target);
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
            "serve" | "server" => {
                if !std::path::Path::new("Cargo.toml").exists() {
                    println!("{}", "❌ Error: File Cargo.toml tidak ditemukan di direktori saat ini.".red().bold());
                    println!("{}", "   Pastikan Anda berada di root project sebelum menjalankan server.".cyan());
                    return;
                }
                let run_android = args.iter().any(|arg| arg == "--android");
                let run_desktop = args.iter().any(|arg| arg == "--desktop");
                if run_android || run_desktop {
                    let _ = dotenv();
                    let cfg = rustbasic_core::Config::load();
                    let vite_port = cfg.vite_port;

                    let vite_already_running = if let Ok(addr) = format!("127.0.0.1:{}", vite_port).parse() {
                        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(100)).is_ok()
                    } else {
                        false
                    };

                    let vite_child = if vite_already_running {
                        println!("📦 Vite Development Server sudah berjalan di port {}.", vite_port);
                        None
                    } else if std::path::Path::new("package.json").exists() {
                        println!("📦 Memulai Vite Development Server (npm run dev)...");
                        let mut vite_cmd = if cfg!(target_os = "windows") {
                            let mut c = std::process::Command::new("cmd");
                            c.args(["/C", "npm", "run", "dev"]);
                            c
                        } else {
                            let mut c = std::process::Command::new("npm");
                            c.args(["run", "dev"]);
                            c
                        };
                        vite_cmd
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                            .ok()
                    } else {
                        None
                    };

                    // Tunggu sebentar agar port Vite mulai terikat jika baru dinyalakan
                    if vite_child.is_some() {
                        std::thread::sleep(std::time::Duration::from_millis(800));
                    }

                    let mut cmd = std::process::Command::new("cargo");
                    cmd.arg("run");
                    cmd.arg("--");
                    cmd.args(&args[1..]);
                    let status = utils::run_cargo_with_progress(cmd);
                    
                    if let Some(mut child) = vite_child {
                        let _ = child.kill();
                        println!("👋 Menghentikan Vite Development Server.");
                    }

                    if let Ok(s) = status {
                        if !s.success() {
                            std::process::exit(s.code().unwrap_or(1));
                        }
                    }
                    return;
                }

                if std::path::Path::new("package.json").exists() {
                    if !std::path::Path::new("node_modules").exists() {
                        println!("\n📦 {}...", "node_modules tidak ditemukan. Menjalankan npm install".cyan().bold());
                        match std::process::Command::new("npm").arg("install").status() {
                            Ok(status) => {
                                if !status.success() {
                                    println!("{}", "❌ Error: npm install gagal.".red().bold());
                                    std::process::exit(status.code().unwrap_or(1));
                                }
                            }
                            Err(e) => {
                                println!("{}", "❌ Error: Gagal mengeksekusi 'npm install'. Pastikan Node.js & npm terinstal.".red().bold());
                                println!("   Detail: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }

                    if !std::path::Path::new("src/dist").exists() {
                        println!("\n📦 {}...", "src/dist tidak ditemukan. Menjalankan npm run build".cyan().bold());
                        match std::process::Command::new("npm").args(["run", "build"]).status() {
                            Ok(status) => {
                                if !status.success() {
                                    println!("{}", "❌ Error: npm run build gagal.".red().bold());
                                    std::process::exit(status.code().unwrap_or(1));
                                }
                            }
                            Err(e) => {
                                println!("{}", "❌ Error: Gagal mengeksekusi 'npm run build'.".red().bold());
                                println!("   Detail: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }

                let has_watch = std::process::Command::new("cargo")
                    .args(["watch", "--version"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if has_watch {
                    println!("\n   {} {}", "🚀".bold(), "Menjalankan server RustBasic dengan Auto-Reload...".magenta().bold());

                    let mut watch_args = vec![
                        "watch".to_string(),
                        "-c".to_string(),
                        "-q".to_string(),
                        "--no-ignore".to_string(),
                        "-i".to_string(),
                        "target".to_string(),
                        "-w".to_string(),
                        "src".to_string(),
                    ];

                    if std::path::Path::new(".env").exists() {
                        watch_args.push("-w".to_string());
                        watch_args.push(".env".to_string());
                    }

                    if std::path::Path::new("src/resources").exists() {
                        watch_args.push("-w".to_string());
                        watch_args.push("src/resources".to_string());
                    }

                    let current_exe = std::env::current_exe()
                        .ok()
                        .and_then(|p| p.to_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "rustbasic".to_string());

                    watch_args.push("-s".to_string());
                    watch_args.push(format!("\"{}\" compile:run", current_exe));

                    let status = std::process::Command::new("cargo")
                        .args(&watch_args)
                        .status()
                        .expect("❌ Gagal menjalankan cargo watch.");
                    
                    if !status.success() {
                        std::process::exit(status.code().unwrap_or(1));
                    }
                } else {
                    println!("\n{} {}", "⚠️  Peringatan:".yellow().bold(), "cargo-watch tidak terdeteksi. Menjalankan server tanpa Auto-Reload...".yellow());
                    println!("{}", "💡 Tips: Instal cargo-watch dengan 'cargo install cargo-watch' untuk mengaktifkan Auto-Reload.".cyan().italic());
                    println!("\n   {} {}", "🚀".bold(), "Menjalankan server RustBasic...".magenta().bold());

                    let mut cmd = std::process::Command::new("cargo");
                    cmd.arg("run");
                    let status = utils::run_cargo_with_progress(cmd)
                        .expect("❌ Gagal menjalankan cargo run.");
                    
                    if !status.success() {
                        std::process::exit(status.code().unwrap_or(1));
                    }
                }
            }
            "migrate" | "migrate:refresh" | "migrate:back" | "migrate:rollback" | "db:seed" | "storage:link" => {
                // Delegasi ke cargo run untuk memastikan kode lokal terbaca
                delegate_to_cargo(&args);
            }
            "cache:clear" => {
                database::clear_cache().await;
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

    match status {
        Ok(s) if s.success() => {
            let git_dir = format!("{}/.git", project_name);
            let git_path = std::path::Path::new(&git_dir);
            if let Err(e) = utils::remove_dir_all_recursive(git_path) {
                println!("{} Gagal menghapus folder .git template: {}", "⚠️  Peringatan:".yellow().bold(), e);
            }
            
            let env_example = format!("{}/.env.example", project_name);
            let env_file = format!("{}/.env", project_name);
            if std::path::Path::new(&env_example).exists() {
                let _ = std::fs::copy(&env_example, &env_file);
            }
            
            // Bersihkan path = "../rustbasic-core" dari Cargo.toml proyek baru
            let cargo_toml_path = format!("{}/Cargo.toml", project_name);
            if let Ok(content) = std::fs::read_to_string(&cargo_toml_path) {
                let cleaned = content
                    .replace(", path = \"../rustbasic-core\"", "")
                    .replace("path = \"../rustbasic-core\", ", "")
                    .replace("path = \"../rustbasic-core\"", "");
                let _ = std::fs::write(&cargo_toml_path, cleaned);
            }
            if std::env::set_current_dir(project_name).is_ok() {
                database::generate_app_key();
                println!("📦 {}", "Mengunduh dependencies...".bold());
                let fetch_status = std::process::Command::new("cargo").args(["fetch"]).status();
                if fetch_status.is_err() || !fetch_status.unwrap().success() {
                    println!("{} Gagal mengunduh dependencies menggunakan cargo fetch. Anda dapat menjalankannya secara manual nanti.", "⚠️  Peringatan:".yellow().bold());
                }
                
                println!("\n✅ {}", "Project berhasil dibuat!".green().bold());
                
                println!("\n🚀 {}", "Untuk memulai pengembangan, ikuti langkah berikut:".magenta().bold());
                
                println!("\n  📂 {}", "1. Masuk ke direktori project:".bold());
                println!("     {} {}", "$".dimmed(), format!("cd {}", project_name).cyan().bold());
                
                println!("\n  💻 {}", "2. Setup & Jalankan Frontend:".bold());
                println!("     {} {}", "$".dimmed(), "npm install".cyan().bold());
                println!("     {} {}    {}", "$".dimmed(), "npm run dev".cyan().bold(), "# Menjalankan development server (Vite/Inertia)".dimmed());
                println!("     {} {} {} {}", "atau".dimmed(), "$".dimmed(), "npm run build".cyan().bold(), "# Build frontend untuk produksi".dimmed());
                
                println!("\n  🦀 {}", "3. Jalankan Backend:".bold());
                println!("     {} {}", "$".dimmed(), "rustbasic serve".cyan().bold());
                println!();
            }
        }
        Ok(s) => {
            println!("\n❌ {} (Status: {})", "Gagal mendownload template project. Git clone keluar dengan error.".red().bold(), s);
        }
        Err(e) => {
            println!("\n❌ {}", "Gagal menjalankan perintah 'git'. Pastikan Git sudah terinstal dan ada dalam PATH sistem Anda.".red().bold());
            println!("   {} {}", "Detail Error:".dimmed(), e);
        }
    }
}

fn print_help() {
    println!("\n{}", "🛠️  RustBasic CLI".magenta().bold());
    println!("{}", "=================".magenta());
    println!("{}", "Penggunaan:".bold());
    println!("  {} {} <Nama>         {}", "rustbasic".blue(), "new".green(), "Membuat project RustBasic baru".dimmed());
    println!("  {} {}                 {}", "rustbasic".blue(), "serve".green(), "Menjalankan server pengembangan (Auto-Reload/Fallback)".dimmed());
    println!("  {} {}                 {}", "rustbasic".blue(), "test".green(), "Menjalankan unit/integration testing proyek".dimmed());
    println!("  {} {}                 {}", "rustbasic".blue(), "build".green(), "Membangun project RustBasic".dimmed());
    
    println!("\n{}", "Database & Migrasi:".bold());
    println!("  {} {}               {}", "rustbasic".blue(), "migrate".green(), "Menjalankan migrasi database".dimmed());
    println!("  {} {}       {}", "rustbasic".blue(), "migrate:refresh".green(), "Mereset dan menjalankan ulang semua migrasi".dimmed());
    println!("  {} {}          {}", "rustbasic".blue(), "migrate:back".green(), "Rollback migrasi database terakhir".dimmed());
    println!("  {} {}               {}", "rustbasic".blue(), "db:seed".green(), "Menjalankan seeder database".dimmed());
    
    println!("\n{}", "Scaffolding & Generator:".bold());
    println!("  {} {} <Nama>   {}", "rustbasic".blue(), "make:controller".green(), "Membuat controller baru".dimmed());
    println!("  {} {} <Nama> [-m]   {}", "rustbasic".blue(), "make:model".green(), "Membuat model baru (tambahkan -m untuk migrasinya)".dimmed());
    println!("  {} {} <Nama>    {}", "rustbasic".blue(), "make:migration".green(), "Membuat file migrasi tabel baru".dimmed());
    println!("  {} {} <Kolom> <Tabel> {}", "rustbasic".blue(), "make:migration:add".green(), "Membuat file migrasi tambah kolom baru".dimmed());
    println!("  {} {} <Nama>     {}", "rustbasic".blue(), "make:middleware".green(), "Membuat middleware baru".dimmed());
    println!("  {} {} <Nama>       {}", "rustbasic".blue(), "make:seeder".green(), "Membuat seeder baru".dimmed());
    println!("  {} {} <Nama> [--unit] {}", "rustbasic".blue(), "make:test".green(), "Membuat unit/feature test baru".dimmed());
    println!("  {} {} <Nama> [--model=<Model>] {}", "rustbasic".blue(), "make:observer".green(), "Membuat observer baru".dimmed());
    println!("  {} {} <Nama>          {}", "rustbasic".blue(), "make:service".green(), "Membuat service baru".dimmed());

    println!("\n{}", "Utilitas & Monitoring:".bold());
    println!("  {} {}          {}", "rustbasic".blue(), "key:generate".green(), "Membuat application key (APP_KEY) baru".dimmed());
    println!("  {} {}           {}", "rustbasic".blue(), "cache:clear".green(), "Membersihkan cache logs dan sessions database".dimmed());
    println!("  {} {}          {}", "rustbasic".blue(), "storage:link".green(), "Menghubungkan folder storage ke folder public".dimmed());
    println!("  {} {}            {}", "rustbasic".blue(), "route:list".green(), "Menampilkan daftar route aktif".dimmed());
    println!("  {} {}        {}", "rustbasic".blue(), "check:security".green(), "Melakukan audit keamanan dependency".dimmed());
    println!("  {} {}          {}", "rustbasic".blue(), "check:update".green(), "Memeriksa pembaruan dependency di crates.io".dimmed());
    println!("  {} {}               {}", "rustbasic".blue(), "version".green(), "Menampilkan versi RustBasic CLI".dimmed());
    println!("  {} {}                 {}", "rustbasic".blue(), "clean".green(), "Membersihkan cache kompilasi Rust dan Docker".dimmed());

    println!("\n{}", "Package Manager:".bold());
    println!("  {} {} <Package>     {}", "rustbasic".blue(), "install".green(), "Install package RustBasic ke project".dimmed());
    println!("  {} {}    {}", "rustbasic".blue(), "list packages".green(), "Tampilkan daftar package yang terinstall".dimmed());
    println!("  {} {} <Package>   {}", "rustbasic".blue(), "uninstall".green(), "Hapus package beserta file konfigurasinya".dimmed());
    println!("  {} {} <Target>    {}", "rustbasic".blue(), "publish".green(), "Salin konfigurasi package (misal: cors, csrf) ke local".dimmed());

    println!("\n{}", "Package tersedia:".bold());
    println!("  {}  {}", "rustbasic-breeze".cyan(), "→ Authentication scaffolding (login, register, reset password)".dimmed());
    
    println!("\n💡 Gunakan 'rustbasic version' untuk melihat informasi versi saat ini.");
}
