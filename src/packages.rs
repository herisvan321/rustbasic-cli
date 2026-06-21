/* ---------------------------------------------------------
 * 📦 LABEL: PACKAGE MANAGER (src/packages.rs)
 * Menangani install, list, dan uninstall package di project
 * RustBasic menggunakan manifest .rustbasic_packages.json
 * --------------------------------------------------------- */

use rustbasic_core::colored::*;
use rustbasic_core::serde::{Deserialize, Serialize};
use rustbasic_core::serde_json;
use std::process::Command;

const MANIFEST_FILE: &str = ".rustbasic_packages.json";

// Registry package yang didukung beserta metadata-nya
struct PackageInfo {
    /// Versi default yang akan digunakan saat install via CLI
    version: &'static str,
    /// Deskripsi singkat package
    description: &'static str,
    /// Command yang dijalankan setelah install (opsional)
    setup_command: Option<&'static str>,
    /// Command yang dijalankan sebelum uninstall (opsional)
    remove_command: Option<&'static str>,
}

fn known_packages(name: &str) -> Option<PackageInfo> {
    match name {
        "rustbasic-breeze" => Some(PackageInfo {
            version: "0.0",
            description: "Authentication scaffolding (login, register, reset password)",
            setup_command: Some("breeze:install"),
            remove_command: Some("breeze:remove"),
        }),
        "rustbasic-activitylog" => Some(PackageInfo {
            version: "0.0",
            description: "Activity logging package for tracking actions and HTTP requests",
            setup_command: Some("activitylog:install"),
            remove_command: Some("activitylog:remove"),
        }),
        "rustbasic-jwt" => Some(PackageInfo {
            version: "0.0",
            description: "JWT authentication package (tokens, claims, blacklist)",
            setup_command: None,
            remove_command: Some("jwt:remove"),
        }),
        "rustbasic-medialibrary" => Some(PackageInfo {
            version: "0.0",
            description: "Advanced media library management (upload, WebP compression, S3 integration)",
            setup_command: None,
            remove_command: None,
        }),
        "rustbasic-permission" => Some(PackageInfo {
            version: "0.0",
            description: "Role and Permission management package (RBAC)",
            setup_command: Some("permission:install"),
            remove_command: Some("permission:remove"),
        }),
        "rustbasic-translatable" => Some(PackageInfo {
            version: "0.0",
            description: "Multi-language JSON translation and localization package",
            setup_command: None,
            remove_command: None,
        }),
        "rustbasic-webp" => Some(PackageInfo {
            version: "0.0",
            description: "High-performance WebP image conversion and resizing package",
            setup_command: None,
            remove_command: None,
        }),
        "rustbasic-native" => Some(PackageInfo {
            version: "0.0",
            description: "Native platform wrapper package for running RustBasic server inside Mobile (Android/iOS) & Desktop apps",
            setup_command: Some("native:install"),
            remove_command: Some("native:remove"),
        }),
        _ => None,
    }
}

// ─── Manifest Structs ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rustbasic_core::serde")]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub installed_at: String,
    pub source: String, // "install" | "manual"
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(crate = "rustbasic_core::serde")]
pub struct PackageManifest {
    pub packages: Vec<InstalledPackage>,
}

// ─── Manifest I/O ─────────────────────────────────────────────────────────────

pub fn read_manifest() -> PackageManifest {
    if let Ok(content) = std::fs::read_to_string(MANIFEST_FILE) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        PackageManifest::default()
    }
}

fn write_manifest(manifest: &PackageManifest) {
    if let Ok(json) = serde_json::to_string_pretty(manifest) {
        std::fs::write(MANIFEST_FILE, json).ok();
    }
}

// ─── Cargo.toml Helpers ───────────────────────────────────────────────────────

fn cargo_toml_path() -> &'static str {
    "Cargo.toml"
}

fn read_cargo_toml() -> Option<String> {
    std::fs::read_to_string(cargo_toml_path()).ok()
}

fn write_cargo_toml(content: &str) {
    std::fs::write(cargo_toml_path(), content).ok();
}

/// Cek apakah package sudah ada di Cargo.toml
fn cargo_has_package(name: &str) -> bool {
    read_cargo_toml()
        .map(|c| c.contains(name))
        .unwrap_or(false)
}

/// Tambahkan dependency ke Cargo.toml
fn cargo_add_package(name: &str, version: &str) -> bool {
    let Some(mut content) = read_cargo_toml() else {
        println!("{}", "❌ Tidak dapat membaca Cargo.toml".red().bold());
        return false;
    };

    if content.contains(name) {
        return true; // sudah ada
    }

    let use_local_path = std::path::Path::new(&format!("../{}", name)).exists();
    let dep_line = if use_local_path {
        format!("{} = {{ path = \"../{}\", version = \"{}\" }}\n", name, name, version)
    } else {
        format!("{} = \"{}\"\n", name, version)
    };

    // Sisipkan setelah [dependencies]
    if let Some(pos) = content.find("[dependencies]") {
        let insert_at = pos + "[dependencies]".len();
        // Cari akhir baris [dependencies]
        let after = &content[insert_at..];
        let newline_pos = after.find('\n').map(|p| insert_at + p + 1).unwrap_or(insert_at);
        content.insert_str(newline_pos, &dep_line);
        write_cargo_toml(&content);
        true
    } else {
        println!("{}", "❌ Tidak dapat menemukan section [dependencies] di Cargo.toml".red().bold());
        false
    }
}

/// Hapus dependency dari Cargo.toml
fn cargo_remove_package(name: &str) {
    if let Some(content) = read_cargo_toml() {
        let filtered: String = content
            .lines()
            .filter(|line| !line.contains(name))
            .collect::<Vec<_>>()
            .join("\n");
        // Pertahankan trailing newline
        let result = if content.ends_with('\n') {
            format!("{}\n", filtered)
        } else {
            filtered
        };
        write_cargo_toml(&result);
    }
}

/// Scan Cargo.toml untuk package rustbasic-* yang belum ada di manifest
pub fn sync_manual_packages(manifest: &mut PackageManifest) {
    let Some(content) = read_cargo_toml() else { return };
    let known_in_manifest: Vec<String> = manifest.packages.iter().map(|p| p.name.clone()).collect();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') { continue; }
        if let Some(idx) = trimmed.find("rustbasic-") {
            let rest = &trimmed[idx..];
            // Ekstrak nama package: ambil sampai karakter non-alfanumerik/dash
            let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '-')
                .unwrap_or(rest.len());
            let pkg_name = &rest[..name_end];

            // Skip core dan cli
            if pkg_name == "rustbasic-core" || pkg_name == "rustbasic-cli" {
                continue;
            }

            if !known_in_manifest.contains(&pkg_name.to_string()) {
                // Ekstrak versi jika ada
                let version = extract_version_from_line(trimmed).unwrap_or_else(|| "?".to_string());
                let desc = known_packages(pkg_name)
                    .map(|p| p.description.to_string())
                    .unwrap_or_else(|| "Package eksternal".to_string());

                manifest.packages.push(InstalledPackage {
                    name: pkg_name.to_string(),
                    version,
                    installed_at: "—".to_string(),
                    source: "manual".to_string(),
                    description: desc,
                });
            }
        }
    }
}

fn extract_version_from_line(line: &str) -> Option<String> {
    // Coba cari 'version = "x.x"' atau '"x.x"' pattern
    if let Some(start) = line.find("version = \"") {
        let rest = &line[start + 11..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    // Bentuk ringkas: package = "x.x"
    if let Some(eq) = line.find(" = \"") {
        let rest = &line[eq + 4..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

// ─── Cargo Build ──────────────────────────────────────────────────────────────

fn run_cargo_build() -> bool {
    println!("   {} Mengunduh dan mengkompilasi dependensi...", "📦".bold());
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    let status = crate::utils::run_cargo_with_progress(cmd);
    match status {
        Ok(s) if s.success() => true,
        Ok(s) => {
            println!("{} cargo build gagal dengan exit code: {}", "❌".red().bold(), s);
            false
        }
        Err(e) => {
            println!("{} Gagal menjalankan cargo build: {}", "❌".red().bold(), e);
            false
        }
    }
}

// ─── Setup / Remove via Temp Binary ───────────────────────────────────────────

fn run_setup_command(package_name: &str, command: &str) {
    let bin_dir = "src/bin";
    std::fs::create_dir_all(bin_dir).ok();
    let script_path = format!("{}/temp_pkg_setup.rs", bin_dir);

    let script = match command {
        "breeze:install" => {
            r#"use rustbasic_core::dotenvy::dotenv;
fn main() {
    rustbasic_core::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            dotenv().ok();
            rustbasic_breeze::make_auth().await;
        });
}
"#.to_string()
        }
        "breeze:remove" => {
            r#"use rustbasic_core::dotenvy::dotenv;
fn main() {
    rustbasic_core::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            dotenv().ok();
            rustbasic_breeze::remove_auth().await;
        });
}
"#.to_string()
        }
        "activitylog:remove" => {
            r##"use std::fs;
use std::path::Path;

fn main() {
    println!("⚙️ Membersihkan scaffolding Activity Log...");
    
    // 1. Hapus model file
    let path = "src/app/models/activity_log.rs";
    if Path::new(path).exists() {
        let _ = fs::remove_file(path);
        println!("   🗑️ Dihapus: {}", path);
    }

    // 2. Bersihkan src/app/models/mod.rs
    let mod_path = "src/app/models/mod.rs";
    if Path::new(mod_path).exists() {
        if let Ok(content) = fs::read_to_string(mod_path) {
            let filtered: Vec<String> = content
                .lines()
                .filter(|line| {
                    !line.contains("pub mod activity_log;") &&
                    !line.contains("pub use activity_log::Model as ActivityLog;")
                })
                .map(|s| s.to_string())
                .collect();
            let _ = fs::write(mod_path, filtered.join("\n") + "\n");
            println!("   📝 Diperbarui: {}", mod_path);
        }
    }

    // 3. Hapus migrations
    let migrations_dir = "database/migrations";
    if let Ok(entries) = fs::read_dir(migrations_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("create_activity_log_table") {
                let path = entry.path();
                let _ = fs::remove_file(&path);
                println!("   🗑️ Dihapus: {}", path.display());
                
                // Bersihkan database/migrations/mod.rs
                let mod_name = name.strip_suffix(".rs").unwrap_or(&name);
                let migrations_mod_path = "database/migrations/mod.rs";
                if Path::new(migrations_mod_path).exists() {
                    if let Ok(content) = fs::read_to_string(migrations_mod_path) {
                        let filtered: Vec<String> = content
                            .lines()
                            .filter(|line| {
                                !line.contains(&format!("pub mod {};", mod_name)) &&
                                !line.contains(&format!("Box::new({}::Migration)", mod_name))
                            })
                            .map(|s| s.to_string())
                            .collect();
                        let _ = fs::write(migrations_mod_path, filtered.join("\n") + "\n");
                    }
                }
            }
        }
    }
}
"##.to_string()
        }
        "jwt:remove" => {
            r##"use std::fs;
use std::path::Path;

fn main() {
    println!("⚙️ Membersihkan scaffolding RustBasic JWT...");
    
    // 1. Hapus model files
    let models = ["user.rs", "jwt_blacklist.rs"];
    for model in &models {
        let path = format!("src/app/models/{}", model);
        if Path::new(&path).exists() {
            let _ = fs::remove_file(&path);
            println!("   🗑️ Dihapus: {}", path);
        }
    }

    // 2. Bersihkan src/app/models/mod.rs
    let mod_path = "src/app/models/mod.rs";
    if Path::new(mod_path).exists() {
        if let Ok(content) = fs::read_to_string(mod_path) {
            let filtered: Vec<String> = content
                .lines()
                .filter(|line| {
                    !line.contains("pub mod user;") &&
                    !line.contains("pub use user::Entity as User;") &&
                    !line.contains("pub use user::Model as User;") &&
                    !line.contains("pub mod jwt_blacklist;") &&
                    !line.contains("pub use jwt_blacklist::Entity as JwtBlacklist;") &&
                    !line.contains("pub use jwt_blacklist::Model as JwtBlacklist;")
                })
                .map(|s| s.to_string())
                .collect();
            let _ = fs::write(mod_path, filtered.join("\n") + "\n");
            println!("   📝 Diperbarui: {}", mod_path);
        }
    }

    // 3. Hapus migrations
    let migrations_dir = "database/migrations";
    if let Ok(entries) = fs::read_dir(migrations_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if (name.contains("create_users_table") && name != "m20260501_000002_create_users_table.rs") || name.contains("create_jwt_blacklists_table") {
                let path = entry.path();
                let _ = fs::remove_file(&path);
                println!("   🗑️ Dihapus: {}", path.display());
                
                // Bersihkan database/migrations/mod.rs
                let mod_name = name.strip_suffix(".rs").unwrap_or(&name);
                let migrations_mod_path = "database/migrations/mod.rs";
                if Path::new(migrations_mod_path).exists() {
                    if let Ok(content) = fs::read_to_string(migrations_mod_path) {
                        let filtered: Vec<String> = content
                            .lines()
                            .filter(|line| {
                                !line.contains(&format!("pub mod {};", mod_name)) &&
                                !line.contains(&format!("Box::new({}::Migration)", mod_name))
                            })
                            .map(|s| s.to_string())
                            .collect();
                        let _ = fs::write(migrations_mod_path, filtered.join("\n") + "\n");
                    }
                }
            }
        }
    }

    // 4. Bersihkan .env dari konfigurasi JWT
    let env_path = ".env";
    if Path::new(env_path).exists() {
        if let Ok(content) = fs::read_to_string(env_path) {
            let filtered: Vec<String> = content
                .lines()
                .filter(|line| {
                    !line.starts_with("JWT_") &&
                    !line.contains("# --- JWT CONFIG ---")
                })
                .map(|s| s.to_string())
                .collect();
            let _ = fs::write(env_path, filtered.join("\n") + "\n");
            println!("   📝 Diperbarui: {}", env_path);
        }
    }
}
"##.to_string()
        }
        "permission:remove" => {
            r##"use std::fs;
use std::path::Path;

fn main() {
    println!("⚙️ Membersihkan scaffolding RBAC Permission...");
    
    // 1. Hapus model files
    let models = [
        "role.rs",
        "permission.rs",
        "model_has_role.rs",
        "model_has_permission.rs",
        "role_has_permission.rs"
    ];
    for model in &models {
        let path = format!("src/app/models/{}", model);
        if Path::new(&path).exists() {
            let _ = fs::remove_file(&path);
            println!("   🗑️ Dihapus: {}", path);
        }
    }

    // 2. Bersihkan src/app/models/mod.rs
    let mod_path = "src/app/models/mod.rs";
    if Path::new(mod_path).exists() {
        if let Ok(content) = fs::read_to_string(mod_path) {
            let filtered: Vec<String> = content
                .lines()
                .filter(|line| {
                    !line.contains("pub mod role;") &&
                    !line.contains("pub use role::Model as Role;") &&
                    !line.contains("pub use role::Entity as Role;") &&
                    !line.contains("pub mod permission;") &&
                    !line.contains("pub use permission::Model as Permission;") &&
                    !line.contains("pub use permission::Entity as Permission;") &&
                    !line.contains("pub mod model_has_role;") &&
                    !line.contains("pub use model_has_role::Model as ModelHasRole;") &&
                    !line.contains("pub use model_has_role::Entity as ModelHasRole;") &&
                    !line.contains("pub mod model_has_permission;") &&
                    !line.contains("pub use model_has_permission::Model as ModelHasPermission;") &&
                    !line.contains("pub use model_has_permission::Entity as ModelHasPermission;") &&
                    !line.contains("pub mod role_has_permission;") &&
                    !line.contains("pub use role_has_permission::Model as RoleHasPermission;") &&
                    !line.contains("pub use role_has_permission::Entity as RoleHasPermission;")
                })
                .map(|s| s.to_string())
                .collect();
            let _ = fs::write(mod_path, filtered.join("\n") + "\n");
            println!("   📝 Diperbarui: {}", mod_path);
        }
    }

    // 3. Hapus migrations
    let migrations_dir = "database/migrations";
    if let Ok(entries) = fs::read_dir(migrations_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("create_rbac_tables") {
                let path = entry.path();
                let _ = fs::remove_file(&path);
                println!("   🗑️ Dihapus: {}", path.display());
                
                // Bersihkan database/migrations/mod.rs
                let mod_name = name.strip_suffix(".rs").unwrap_or(&name);
                let migrations_mod_path = "database/migrations/mod.rs";
                if Path::new(migrations_mod_path).exists() {
                    if let Ok(content) = fs::read_to_string(migrations_mod_path) {
                        let filtered: Vec<String> = content
                            .lines()
                            .filter(|line| {
                                !line.contains(&format!("pub mod {};", mod_name)) &&
                                !line.contains(&format!("Box::new({}::Migration)", mod_name))
                            })
                            .map(|s| s.to_string())
                            .collect();
                        let _ = fs::write(migrations_mod_path, filtered.join("\n") + "\n");
                    }
                }
            }
        }
    }
}
"##.to_string()
        }
        "activitylog:install" => {
            let use_local = std::path::Path::new("../rustbasic-activitylog").exists();
            let cargo_args = if use_local {
                "[\"run\", \"--manifest-path\", \"../rustbasic-activitylog/Cargo.toml\", \"--\", \"install\"]"
            } else {
                "[\"run\", \"-p\", \"rustbasic-activitylog\", \"--\", \"install\"]"
            };
            format!(
                r#"fn main() {{
    println!("⚙️ Menjalankan generator scaffolding Activity Log...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.args({});
    if let Ok(status) = cmd.status() {{
        if !status.success() {{
            eprintln!("❌ Scaffolding Activity Log gagal");
        }}
    }}
}}
"#,
                cargo_args
            )
        }
        "permission:install" => {
            let use_local = std::path::Path::new("../rustbasic-permission").exists();
            let cargo_args = if use_local {
                "[\"run\", \"--manifest-path\", \"../rustbasic-permission/Cargo.toml\", \"--\", \"install\"]"
            } else {
                "[\"run\", \"-p\", \"rustbasic-permission\", \"--\", \"install\"]"
            };
            format!(
                r#"fn main() {{
    println!("⚙️ Menjalankan generator scaffolding RBAC Permission...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.args({});
    if let Ok(status) = cmd.status() {{
        if !status.success() {{
            eprintln!("❌ Scaffolding RBAC Permission gagal");
        }}
    }}
}}
"#,
                cargo_args
            )
        }
        "native:install" => {
            let use_local = std::path::Path::new("../rustbasic-native").exists();
            let cargo_args = if use_local {
                "[\"run\", \"--manifest-path\", \"../rustbasic-native/Cargo.toml\", \"--\", \"install\"]"
            } else {
                "[\"run\", \"-p\", \"rustbasic-native\", \"--\", \"install\"]"
            };
            format!(
                r#"fn main() {{
    println!("⚙️ Menjalankan generator scaffolding RustBasic Native...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.args({});
    if let Ok(status) = cmd.status() {{
        if !status.success() {{
            eprintln!("❌ Scaffolding RustBasic Native gagal");
        }}
    }}
}}
"#,
                cargo_args
            )
        }
        "native:remove" => {
            let use_local = std::path::Path::new("../rustbasic-native").exists();
            let cargo_args = if use_local {
                "[\"run\", \"--manifest-path\", \"../rustbasic-native/Cargo.toml\", \"--\", \"uninstall\"]"
            } else {
                "[\"run\", \"-p\", \"rustbasic-native\", \"--\", \"uninstall\"]"
            };
            format!(
                r#"fn main() {{
    println!("⚙️ Membersihkan scaffolding RustBasic Native...");
    let mut cmd = std::process::Command::new("cargo");
    cmd.args({});
    if let Ok(status) = cmd.status() {{
        if !status.success() {{
            eprintln!("❌ Pembersihan scaffolding RustBasic Native gagal");
        }}
    }}
}}
"#,
                cargo_args
            )
        }
        _ => return,
    };

    std::fs::write(&script_path, &script).ok();

    println!("   {} Menjalankan setup {}...", "⚙️".bold(), package_name.cyan().bold());
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "temp_pkg_setup"]);
    let status = crate::utils::run_cargo_with_progress(cmd);

    // Cleanup script
    std::fs::remove_file(&script_path).ok();
    // Hapus folder bin jika kosong
    if let Ok(entries) = std::fs::read_dir(bin_dir)
        && entries.count() == 0 {
        std::fs::remove_dir(bin_dir).ok();
    }

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => println!("{} Setup command gagal (exit {})", "⚠️".yellow().bold(), s),
        Err(e) => println!("{} Gagal menjalankan setup: {}", "⚠️".yellow().bold(), e),
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Install sebuah package ke project
pub fn install_package(name: &str) {
    println!("\n{} {}", "📦 Installing:".magenta().bold(), name.cyan().bold());

    // 1. Cek apakah package sudah terinstall
    let mut manifest = read_manifest();
    if manifest.packages.iter().any(|p| p.name == name) {
        println!("{} Package '{}' sudah terinstall.", "⚠️".yellow().bold(), name.yellow());
        println!("   Gunakan '{}' untuk melihat daftar package.", "rustbasic list packages".cyan());
        return;
    }

    // 2. Tentukan versi
    let (version, description, setup_cmd) = if let Some(info) = known_packages(name) {
        (info.version.to_string(), info.description.to_string(), info.setup_command.map(|s| s.to_string()))
    } else {
        println!("{} Package '{}' tidak dikenali dalam registry RustBasic.", "⚠️".yellow().bold(), name.yellow());
        println!("   Gunakan versi spesifik dengan menambahkan ke Cargo.toml secara manual.");
        return;
    };

    // 3. Tambahkan ke Cargo.toml
    println!("   {} Menambahkan ke Cargo.toml...", "📝".bold());
    if !cargo_add_package(name, &version) {
        return;
    }

    // 4. cargo build
    if !run_cargo_build() {
        // Rollback
        cargo_remove_package(name);
        return;
    }

    // 5. Jalankan setup function
    if let Some(cmd) = &setup_cmd {
        run_setup_command(name, cmd);
    }

    // 6. Catat di manifest
    let now = rustbasic_core::chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    manifest.packages.push(InstalledPackage {
        name: name.to_string(),
        version,
        installed_at: now,
        source: "install".to_string(),
        description,
    });
    write_manifest(&manifest);

    println!("\n{} Package '{}' berhasil diinstall!", "✅".green().bold(), name.cyan().bold());
    println!("   Gunakan '{}' untuk melihat daftar package.", "rustbasic list packages".cyan());
}

/// Tampilkan daftar package yang terinstall
pub fn list_packages() {
    let mut manifest = read_manifest();
    sync_manual_packages(&mut manifest);

    println!("\n{}", "📦 RustBasic Package Manager".magenta().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════════════════".magenta());

    if manifest.packages.is_empty() {
        println!("{}", "   Belum ada package tambahan yang terinstall.".dimmed());
        println!("   Gunakan '{}' untuk menginstall package.", "rustbasic install <nama-package>".cyan());
    } else {
        // Header tabel
        println!(
            "  {:<28} {:<10} {:<10} {:<22} {}",
            "PACKAGE".bold(),
            "VERSION".bold(),
            "SOURCE".bold(),
            "INSTALLED AT".bold(),
            "DESCRIPTION".bold()
        );
        println!("{}", "  ─────────────────────────────────────────────────────────────────────".dimmed());

        for pkg in &manifest.packages {
            let source_display = match pkg.source.as_str() {
                "manual" => "manual".yellow().to_string(),
                _ => "install".green().to_string(),
            };
            let installed_at = if pkg.installed_at == "—" {
                "—".dimmed().to_string()
            } else {
                pkg.installed_at.clone().dimmed().to_string()
            };
            println!(
                "  {:<28} {:<10} {:<18} {:<22} {}",
                pkg.name.cyan(),
                pkg.version,
                source_display,
                installed_at,
                pkg.description.dimmed()
            );
        }
    }

    println!("{}", "═══════════════════════════════════════════════════════════════════════════".magenta());
    println!(
        "  {} Total: {} package\n",
        "📊".bold(),
        manifest.packages.len().to_string().cyan().bold()
    );
}

/// Uninstall sebuah package dari project
pub fn uninstall_package(name: &str) {
    println!("\n{} {}", "🗑️  Uninstalling:".red().bold(), name.cyan().bold());

    let mut manifest = read_manifest();
    let pkg_idx = manifest.packages.iter().position(|p| p.name == name);

    // Cek jika tidak ada di manifest tapi ada di Cargo.toml
    let in_cargo = cargo_has_package(name);
    if pkg_idx.is_none() && !in_cargo {
        println!("{} Package '{}' tidak ditemukan.", "❌".red().bold(), name.yellow());
        return;
    }

    // 1. Jalankan remove function (cleanup file scaffolding)
    if let Some(info) = known_packages(name)
        && let Some(remove_cmd) = info.remove_command {
        // Pastikan package ada di Cargo.toml sebelum menjalankan remove
        if in_cargo {
            run_setup_command(name, remove_cmd);
        }
    }

    // 2. Hapus dari Cargo.toml
    println!("   {} Menghapus dari Cargo.toml...", "📝".bold());
    cargo_remove_package(name);

    // 3. cargo build untuk update lock file
    println!("   {} Memperbarui dependencies...", "📦".bold());
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    let _ = crate::utils::run_cargo_with_progress(cmd);

    // 4. Hapus dari manifest
    if let Some(idx) = pkg_idx {
        manifest.packages.remove(idx);
        write_manifest(&manifest);
    }

    println!("\n{} Package '{}' berhasil diuninstall!", "✅".green().bold(), name.cyan().bold());
}
