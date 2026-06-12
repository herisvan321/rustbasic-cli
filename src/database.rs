use std::fs;
use rustbasic_core::base64::engine::general_purpose;
use rustbasic_core::sql::{self, AnyPool};
use rustbasic_core::colored::*;
use rustbasic_core::rand;
use rustbasic_core::regex::Regex;
use rustbasic_core::dotenvy;

pub struct LocalDbConfig {
    pub db_connection: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_database: String,
    pub db_username: String,
    pub db_password: String,
}

impl LocalDbConfig {
    pub fn load() -> Self {
        use std::env;
        Self {
            db_connection: env::var("DB_CONNECTION").unwrap_or_else(|_| "sqlite".to_string()),
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            db_port: env::var("DB_PORT")
                .unwrap_or_else(|_| "3306".to_string())
                .parse()
                .unwrap_or(3306),
            db_database: env::var("DB_DATABASE").unwrap_or_else(|_| "rustbasic".to_string()),
            db_username: env::var("DB_USERNAME").unwrap_or_else(|_| "root".to_string()),
            db_password: env::var("DB_PASSWORD").unwrap_or_default(),
        }
    }

    pub fn db_url(&self) -> String {
        if self.db_connection == "mysql" {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                self.db_username, self.db_password, self.db_host, self.db_port, self.db_database
            )
        } else {
            format!("sqlite:database/{}.sqlite?mode=rwc", self.db_database)
        }
    }
}

pub async fn connect() -> AnyPool {
    sql::any::install_default_drivers();

    let cfg = LocalDbConfig::load();
    let db_url = cfg.db_url();

    if cfg.db_connection != "mysql" {
        let _ = std::fs::create_dir_all("database");
    }

    match AnyPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(e) => {
            let err_msg = e.to_string();
            #[cfg(feature = "mysql")]
            if (err_msg.contains("1049") || err_msg.contains("Unknown database")) && cfg.db_connection == "mysql" {
                println!("{}", "⚠️  Database tidak ditemukan. Mencoba membuat database baru...".yellow());
                let root_url = format!(
                    "mysql://{}:{}@{}:{}",
                    cfg.db_username, cfg.db_password, cfg.db_host, cfg.db_port
                );
                if let Ok(pool) = sql::MySqlPool::connect(&root_url).await {
                    let create_query = format!("CREATE DATABASE IF NOT EXISTS `{}`", cfg.db_database);
                    if sql::query(&create_query).execute(&pool).await.is_ok() {
                        println!("✅ Database '{}' berhasil dibuat.", cfg.db_database.green());
                        return AnyPool::connect(&db_url).await.expect("Gagal terhubung setelah membuat database");
                    }
                }
            }
            #[cfg(not(feature = "mysql"))]
            let _ = err_msg;
            panic!("Gagal terhubung ke database: {:?}", e);
        }
    }
}

pub async fn clear_cache() {
    println!("\n{}", "🧹 Cleaning Cache & Logs...".magenta().bold());

    // 1. Clear Logs
    let log_dir = "storage/logs";
    if let Ok(entries) = fs::read_dir(log_dir) {
        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&path);
                count += 1;
            }
        }
        println!("   {} Folder storage/logs telah dikosongkan. ({} file dibersihkan)", "✅ Logs:".green(), count);
    } else {
        println!("   {} Folder storage/logs tidak ditemukan.", "⚠️  Logs:".yellow());
    }

    // 2. Clear Sessions in DB
    let _ = dotenvy::dotenv();
    let cfg = LocalDbConfig::load();
    let pool = connect().await;

    let sql = if cfg.db_connection == "mysql" {
        "TRUNCATE TABLE sessions"
    } else {
        "DELETE FROM sessions"
    };

    match sql::query(sql).execute(&pool).await {
        Ok(_) => println!("   {} Tabel sessions telah dikosongkan.", "✅ Sessions:".green()),
        Err(e) => println!("   {} Gagal membersihkan tabel sessions. ({})", "❌ Error:".red(), e),
    }

    println!("\n{}", "✨ Cache berhasil dibersihkan!".green().bold());
}

pub fn generate_app_key() {
    println!("\n{}", "🔑 Generating Application Key...".magenta().bold());

    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);

    let encoded = general_purpose::STANDARD.encode(key);
    let key_str = format!("base64:{}", encoded);

    let env_path = ".env";
    match fs::read_to_string(env_path) {
        Ok(content) => {
            let re = Regex::new(r"(?m)^APP_KEY=.*").unwrap();
            let new_content = if re.is_match(&content) {
                re.replace(&content, &format!("APP_KEY={}", key_str)).to_string()
            } else {
                format!("{}\nAPP_KEY={}", content.trim_end(), key_str)
            };

            if let Err(e) = fs::write(env_path, new_content) {
                println!("{} Gagal menulis ke file .env: {}", "❌ Error:".red(), e);
            } else {
                println!("{} {}", "✅ Application key set successfully:".green(), key_str.cyan());
                println!("{}", "💡 Pastikan untuk tidak membagikan APP_KEY ini ke publik!".dimmed());
            }
        }
        Err(_) => {
            println!("{} File .env tidak ditemukan.", "❌ Error:".red());
        }
    }
}

pub async fn ensure_session() {
    if !std::path::Path::new(".env").exists() {
        return;
    }

    let _ = dotenvy::dotenv();
    let cfg = LocalDbConfig::load();
    let db_url = cfg.db_url();

    if cfg.db_connection != "mysql" {
        let _ = std::fs::create_dir_all("database");
    }

    sql::any::install_default_drivers();
    let pool = match AnyPool::connect(&db_url).await {
        Ok(p) => p,
        Err(_) => return,
    };

    // Pastikan tabel sessions ada
    let create_table_sql = if cfg.db_connection == "mysql" {
        "CREATE TABLE IF NOT EXISTS sessions (
            id VARCHAR(255) PRIMARY KEY,
            payload VARCHAR(8000) NOT NULL,
            last_activity BIGINT NOT NULL,
            ip_address VARCHAR(45)
        )"
    } else {
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            payload TEXT NOT NULL,
            last_activity BIGINT NOT NULL,
            ip_address TEXT
        )"
    };

    if sql::query(create_table_sql).execute(&pool).await.is_err() {
        return;
    }

    let session_file = ".rustbasic_session";
    let mut session_id = String::new();
    let mut need_to_write = false;

    if let Ok(content) = fs::read_to_string(session_file) {
        session_id = content.trim().to_string();
    }

    let mut session_exists = false;

    if !session_id.is_empty()
        && let Ok(Some(_)) = sql::query("SELECT id FROM sessions WHERE id = ?")
            .bind(&session_id)
            .fetch_optional(&pool)
            .await
    {
        session_exists = true;
    }

    if !session_exists {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        session_id = general_purpose::STANDARD.encode(bytes);
        need_to_write = true;

        let now = rustbasic_core::chrono::Utc::now().timestamp();
        if sql::query("INSERT INTO sessions (id, payload, last_activity, ip_address) VALUES (?, ?, ?, ?)")
            .bind(&session_id)
            .bind("{}")
            .bind(now)
            .bind("127.0.0.1")
            .execute(&pool)
            .await
            .is_ok()
        {
            println!("✨ {} ({})", "Session baru berhasil dibuat dan disimpan di database.".green().bold(), session_id.cyan());
        }
    } else {
        let now = rustbasic_core::chrono::Utc::now().timestamp();
        let _ = sql::query("UPDATE sessions SET last_activity = ? WHERE id = ?")
            .bind(now)
            .bind(&session_id)
            .execute(&pool)
            .await;
    }

    if need_to_write {
        let _ = fs::write(session_file, &session_id);
    }
}
