use std::fs;
use std::path::Path;
use rustbasic_core::colored::*;

pub fn publish_config(target: &str) {
    let mut selected_target = target.to_string();
    if selected_target.is_empty() {
        println!("🛠️  RustBasic Configuration Publisher");
        println!("Pilih konfigurasi yang ingin dipublikasikan ke proyek Anda:");
        println!("  [1] CORS (Cross-Origin Resource Sharing)");
        println!("  [2] CSRF (Cross-Site Request Forgery)");
        println!("  [3] APP (Application settings & Storage path overrides)");
        let choice = crate::utils::prompt_choice("👉 Pilih nomor konfigurasi (1-3): ", 1, 3);
        match choice {
            1 => selected_target = "cors".to_string(),
            2 => selected_target = "csrf".to_string(),
            3 => selected_target = "app".to_string(),
            _ => return,
        }
    }

    match selected_target.as_str() {
        "cors" => {
            let path = Path::new("src/config/cors.rs");
            if path.exists() {
                println!("ℹ️  File cors.rs sudah ada di src/config/cors.rs");
                return;
            }
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let content = r#"/* ---------------------------------------------------------
 * 📑 LABEL: CORS CONFIGURATION (src/config/cors.rs)
 * Berkas konfigurasi tambahan untuk kustomisasi CORS.
 * --------------------------------------------------------- */

pub struct CorsConfig {
    pub allowed_origins: Vec<&'static str>,
    pub allowed_methods: Vec<&'static str>,
    pub allowed_headers: Vec<&'static str>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*"],
            allowed_methods: vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"],
            allowed_headers: vec!["*"],
        }
    }
}
"#;
            if fs::write(path, content).is_ok() {
                println!("{} {}", "✅ Berhasil mempublikasikan konfigurasi CORS ke:".green().bold(), path.display().to_string().cyan());
                println!("💡 File ini sekarang dapat diimpor untuk menyesuaikan aturan CORS lokal.");
            } else {
                println!("❌ Gagal menulis file CORS config.");
            }
        }
        "csrf" => {
            let path = Path::new("src/config/csrf.rs");
            if path.exists() {
                println!("ℹ️  File csrf.rs sudah ada di src/config/csrf.rs");
                return;
            }
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let content = r#"/* ---------------------------------------------------------
 * 📑 LABEL: CSRF CONFIGURATION (src/config/csrf.rs)
 * Berkas konfigurasi tambahan untuk perlindungan CSRF.
 * --------------------------------------------------------- */

pub struct CsrfConfig {
    pub except_paths: Vec<&'static str>,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            except_paths: vec![], // Masukkan rute yang dikecualikan dari CSRF di sini
        }
    }
}
"#;
            if fs::write(path, content).is_ok() {
                println!("{} {}", "✅ Berhasil mempublikasikan konfigurasi CSRF ke:".green().bold(), path.display().to_string().cyan());
                println!("💡 File ini sekarang dapat diimpor untuk mengecualikan rute tertentu dari CSRF.");
            } else {
                println!("❌ Gagal menulis file CSRF config.");
            }
        }
        "app" => {
            let path = Path::new("src/config/app.rs");
            if path.exists() {
                println!("ℹ️  File app.rs sudah ada di src/config/app.rs");
                return;
            }
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let content = r#"/* ---------------------------------------------------------
 * 📑 LABEL: APP CONFIGURATION (src/config/app.rs)
 * Berkas konfigurasi tambahan untuk kustomisasi lokasi storage lokal.
 * --------------------------------------------------------- */

pub const STORAGE_TARGET: &str = "public/storage";
pub const STORAGE_SOURCE: &str = "storage/app/public";
"#;
            if fs::write(path, content).is_ok() {
                println!("{} {}", "✅ Berhasil mempublikasikan konfigurasi APP ke:".green().bold(), path.display().to_string().cyan());
            } else {
                println!("❌ Gagal menulis file APP config.");
            }
        }
        _ => {
            println!("❌ Target '{}' tidak dikenal untuk di-publish.", selected_target);
            println!("💡 Target yang didukung: cors, csrf, app");
        }
    }
}
