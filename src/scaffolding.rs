use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use rustbasic_core::chrono::Local;
use rustbasic_core::colored::*;
use crate::utils::{to_snake_case, to_pascal_case};

pub fn make_controller(name: &str) {
    let pascal_name = to_pascal_case(name).replace("Controller", "");
    let snake_name = to_snake_case(&pascal_name);
    let class_name = format!("{}Controller", pascal_name);
    let file_name = format!("{}_controller.rs", snake_name);
    let file_path = format!("src/app/http/controllers/{}", file_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Controller".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let template = format!(
r#"/* ---------------------------------------------------------
 * 📑 LABEL: {class_name} ({file_name})
 * --------------------------------------------------------- */

use crate::app::inertia::inertia;
use rustbasic_core::requests::Request;
use rustbasic_core::router::Response;
use rustbasic_core::serde_json::json;

pub struct {class_name};

impl {class_name} {{
    pub async fn index(req: Request) -> Response {{
        inertia(&req, "{pascal_name}", json!({{
            "title": "{class_name}"
        }}))
    }}
}}
"#, class_name = class_name, file_name = file_name, pascal_name = pascal_name);

    fs::write(&file_path, template).expect("Gagal membuat file controller");
    println!("{} {}", "✅ Controller dibuat:".green(), file_path.cyan());

    update_controller_mod_rs(&file_name.replace(".rs", ""));
}

pub fn update_controller_mod_rs(mod_name: &str) {
    let mod_path = "src/app/http/controllers/mod.rs";
    let mut content = String::new();
    if let Ok(mut file) = fs::File::open(mod_path) {
        file.read_to_string(&mut content).ok();
    }

    let line = format!("pub mod {};", mod_name);
    if content.contains(&line) {
        return;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(mod_path)
        .expect("Gagal membuka controllers/mod.rs");

    writeln!(file, "{}", line).ok();
    println!("{} {}", "📝".blue(), "controllers/mod.rs diperbarui.".dimmed());
}

pub fn make_middleware(name: &str) {
    let snake_name = to_snake_case(name).replace("_middleware", "");
    let fn_name = format!("{}_middleware", snake_name);
    let file_name = format!("{}.rs", snake_name);
    let file_path = format!("src/app/http/middleware/{}", file_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Middleware".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let template = format!(
r#"/* ---------------------------------------------------------
 * 📑 LABEL: {label} (middleware/{file_name})
 * --------------------------------------------------------- */

use rustbasic_core::middleware::Next;
use rustbasic_core::requests::Request;
use rustbasic_core::router::Response;

pub async fn {fn_name}(
    req: Request,
    next: Next,
) -> Response {{
    // Lakukan sesuatu sebelum request sampai ke controller
    
    let response = next.run(req).await;
    
    // Lakukan sesuatu setelah request selesai diproses
    
    response
}}
"#, label = name.to_uppercase(), file_name = file_name, fn_name = fn_name);

    fs::write(&file_path, template).expect("Gagal membuat file middleware");
    println!("{} {}", "✅ Middleware dibuat:".green(), file_path.cyan());

    update_middleware_mod_rs(&snake_name);
}

pub fn update_middleware_mod_rs(mod_name: &str) {
    let mod_path = "src/app/http/middleware/mod.rs";
    let mut content = String::new();
    if let Ok(mut file) = fs::File::open(mod_path) {
        file.read_to_string(&mut content).ok();
    }

    let line = format!("pub mod {};", mod_name);
    if content.contains(&line) {
        return;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(mod_path)
        .expect("Gagal membuka middleware/mod.rs");

    writeln!(file, "{}", line).ok();
    println!("{} {}", "📝".blue(), "middleware/mod.rs diperbarui.".dimmed());
}

pub fn make_model(name: &str) {
    let snake_name = to_snake_case(name);
    let table_name = format!("{}s", snake_name);
    let file_path = format!("src/app/models/{}.rs", snake_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Model".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let template = format!(
r#"use rustbasic_core::model;

model! {{
    table: "{table_name}",
    Model {{
        pub id: i32,
        // tambahkan field lainnya di sini
        
        // Contoh Cast Boolean (0/1 dari DB dikonversi otomatis ke bool di Rust):
        // #[serde(default, deserialize_with = "rustbasic_core::support::casts::deserialize_bool", serialize_with = "rustbasic_core::support::casts::serialize_bool")]
        // pub is_active: bool,
        
        // Contoh Cast JSON (Kolom TEXT berisi JSON di DB dikonversi otomatis ke serde_json::Value di Rust):
        // #[serde(default, deserialize_with = "rustbasic_core::support::casts::deserialize_option_json", serialize_with = "rustbasic_core::support::casts::serialize_option_json")]
        // pub metadata: Option<rustbasic_core::serde_json::Value>,
    }}
}}
"#, table_name = table_name);

    fs::write(&file_path, template).expect("Gagal membuat file model");
    println!("{} {}", "✅ Model dibuat:".green(), file_path.cyan());

    update_mod_rs(&to_pascal_case(name), &snake_name);
}

pub fn update_mod_rs(_class_name: &str, snake_name: &str) {
    let mod_path = "src/app/models/mod.rs";
    let mut content = String::new();
    if let Ok(mut file) = fs::File::open(mod_path) {
        file.read_to_string(&mut content).ok();
    }

    let mod_line = format!("pub mod {};", snake_name);
    if content.contains(&mod_line) {
        return;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(mod_path)
        .expect("Gagal membuka models/mod.rs");

    writeln!(file, "{}", mod_line).ok();
    // Tidak ada lagi pub use Entity karena model! macro sudah menggenerate type alias
    
    println!("{} {}", "📝".blue(), "models/mod.rs diperbarui.".dimmed());
}

pub fn make_rust_migration(name: &str) {
    let snake_name = to_snake_case(name);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let mod_name = format!("m{}_{}", timestamp, snake_name);
    let file_path = format!("database/migrations/{}.rs", mod_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Migration".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let table_name = if snake_name.ends_with('s') { snake_name.clone() } else { format!("{}s", snake_name) };

    let template = format!(
r#"use rustbasic_core::{{Schema, SchemaManager, MigrationTrait, DbErr}};
use rustbasic_core::async_trait;

pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {{
    fn name(&self) -> &str {{
        "{mod_name}"
    }}

    async fn up<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::create(manager, "{table_name}", |table| {{
            table.id();
            // table.string("title").not_null();
            table.timestamps();
        }}).await
    }}

    async fn down<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::drop(manager, "{table_name}").await
    }}
}}
"#, table_name = table_name, mod_name = mod_name);

    fs::write(&file_path, template).expect("Gagal membuat file migration");
    println!("{} {}", "✅ Migration Rust dibuat:".green(), file_path.cyan());

    update_migration_mod_rs(&mod_name);
}

pub fn make_rust_migration_add(column: &str, table: &str) {
    let col_snake = to_snake_case(column);
    let table_snake = to_snake_case(table);
    let name = format!("add_{}_to_{}", col_snake, table_snake);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let mod_name = format!("m{}_{}", timestamp, name);
    let file_path = format!("database/migrations/{}.rs", mod_name);

    let template = format!(
r#"use rustbasic_core::{{Schema, SchemaManager, MigrationTrait, DbErr}};
use rustbasic_core::async_trait;

pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {{
    fn name(&self) -> &str {{
        "{mod_name}"
    }}

    async fn up<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::table(manager, "{table_snake}", |table| {{
            table.string("{col_snake}").nullable();
        }}).await
    }}

    async fn down<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::table(manager, "{table_snake}", |table| {{
            table.drop_column("{col_snake}");
        }}).await
    }}
}}
"#, table_snake = table_snake, col_snake = col_snake, mod_name = mod_name);

    fs::write(&file_path, template).expect("Gagal membuat file migration");
    println!("{} {}", "✅ Migration Add dibuat:".green(), file_path.cyan());

    update_migration_mod_rs(&mod_name);
}

pub fn update_migration_mod_rs(mod_name: &str) {
    let mod_path = "database/migrations/mod.rs";
    let mut content = String::new();
    if let Ok(mut file) = fs::File::open(mod_path) {
        file.read_to_string(&mut content).ok();
    }

    // Tambahkan mod declaration
    if !content.contains(&format!("pub mod {};", mod_name)) {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("pub mod {};\n", mod_name));
    }

    // Tambahkan ke list migrations (pattern baru MigratorTrait)
    let search_patterns = [
        "fn migrations() -> Vec<Box<dyn MigrationTrait>> {",
        "fn migrations()->Vec<Box<dyn MigrationTrait>>{",
    ];
    for pattern in &search_patterns {
        if let Some(_pos) = content.find(pattern) {
            let insert_pos = content.find("        ]").unwrap_or(content.len());
            content.insert_str(insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
            break;
        }
    }

    fs::write(mod_path, content).expect("Gagal memperbarui database/migrations/mod.rs");
    println!("{} {}", "📝".blue(), "database/migrations/mod.rs diperbarui.".dimmed());
}

pub fn make_seeder(name: &str) {
    let pascal_name = to_pascal_case(name).replace("Seeder", "");
    let snake_name = to_snake_case(&pascal_name);
    let class_name = format!("{}Seeder", pascal_name);
    let file_name = format!("{}_seeder.rs", snake_name);
    let file_path = format!("database/seeders/{}", file_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Seeder".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let template = format!(
r#"use rustbasic_core::seeder;
use rustbasic_core::colored::Colorize;
// use crate::app::models::{pascal_name}; // Sesuaikan dengan nama model/struct Entity Anda

seeder! {{
    {class_name},
    run(_db) {{
        println!("   {{}} Sedang memproses {class_name}...", "⏳".blue());
        
        // Contoh Penggunaan:
        /*
        {pascal_name}::create(_db, rustbasic_core::serde_json::json!({{
            "name": "Example Data",
        }})).await?;
        */

        Ok(())
    }}
}}
"#, class_name = class_name, pascal_name = pascal_name);

    fs::write(&file_path, template).expect("Gagal membuat file seeder");
    println!("{} {}", "✅ Seeder dibuat:".green(), file_path.cyan());

    update_seeder_mod_rs(&class_name, &file_name.replace(".rs", ""));
}

pub fn update_seeder_mod_rs(class_name: &str, mod_name: &str) {
    // 1. Update database/seeders/mod.rs (mod declaration)
    let db_mod_path = "database/seeders/mod.rs";
    let mut db_content = fs::read_to_string(db_mod_path).expect("Gagal membaca seeders/mod.rs");
    let mod_line = format!("pub mod {};", mod_name);
    if !db_content.contains(&mod_line) {
        db_content.push_str(&format!("{}\n", mod_line));
        fs::write(db_mod_path, db_content).ok();
    }

    // 2. Update src/app/seeder.rs (registration)
    let config_path = "src/app/seeder.rs";
    let mut config_content = fs::read_to_string(config_path).expect("Gagal membaca src/app/seeder.rs");
    let search_pattern = "let seeders: Vec<Box<dyn SeederTrait>> = vec![";
    if let Some(pos) = config_content.find(search_pattern) {
        let insert_pos = pos + search_pattern.len();
        config_content.insert_str(insert_pos, &format!("\n        Box::new(seeders::{}::{}),", mod_name, class_name));
        fs::write(config_path, config_content).ok();
    }
    
    println!("{} {}", "📝".blue(), "Pengaturan seeder diperbarui.".dimmed());
}

pub fn make_test(name: &str, is_unit: bool) {
    let pascal_name = to_pascal_case(name).replace("Test", "");
    let snake_name = to_snake_case(&pascal_name);
    let prefix = if is_unit { "unit" } else { "feature" };
    let file_name = format!("{}_{}_test.rs", prefix, snake_name);
    let file_path = format!("tests/{}", file_name);

    if !std::path::Path::new("tests").exists() {
        fs::create_dir_all("tests").expect("Gagal membuat folder tests");
    }

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Test".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let template = if is_unit {
        format!(
r#"/* ---------------------------------------------------------
 * 🧪 UNIT TEST: {pascal_name} (tests/{file_name})
 * --------------------------------------------------------- */

#[test]
fn test_{snake_name}_logic() {{
    // Contoh asersi sederhana
    let expected = 42;
    let actual = 40 + 2;
    
    assert_eq!(expected, actual, "Fungsi kalkulasi tidak sesuai");
}}
"#, pascal_name = pascal_name, file_name = file_name, snake_name = snake_name)
    } else {
        format!(
r#"/* ---------------------------------------------------------
 * 🧪 FEATURE TEST: {pascal_name} (tests/{file_name})
 * --------------------------------------------------------- */

use rustbasic_core::testing::TestClient;
use rustbasic_core::Config;

#[tokio::test]
async fn test_{snake_name}_page() {{
    // 1. Muat konfigurasi
    let cfg = Config::load();
    
    // 2. Bangun router aplikasi
    let router = rustbasic::routes::build_router();
    
    // 3. Setup TestClient in-memory
    let client = TestClient::new(cfg, router).await;
    
    // 4. Kirim request ke endpoint (misalnya '/')
    let response = client.get("/").await;
    
    // 5. Asersi response status & konten
    response.assert_status(200);
}}
"#, pascal_name = pascal_name, file_name = file_name, snake_name = snake_name)
    };

    fs::write(&file_path, template).expect("Gagal membuat file test");
    println!("{} {}", "✅ Test dibuat:".green(), file_path.cyan());
}

pub fn make_observer(name: &str, model_name: Option<&str>) {
    let pascal_name = to_pascal_case(name).replace("Observer", "");
    let snake_name = to_snake_case(&pascal_name);
    let class_name = format!("{}Observer", pascal_name);
    let file_name = format!("{}_observer.rs", snake_name);
    
    // Ensure src/app/observers/ directory exists
    let dir_path = "src/app/observers";
    fs::create_dir_all(dir_path).expect("Gagal membuat folder observers");
    
    let file_path = format!("{}/{}", dir_path, file_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Observer".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let model_pascal = model_name.map(to_pascal_case).unwrap_or_else(|| pascal_name.clone());
    let mut model_snake = to_snake_case(&model_pascal);
    if !std::path::Path::new(&format!("src/app/models/{}.rs", model_snake)).exists()
        && std::path::Path::new(&format!("src/app/models/{}s.rs", model_snake)).exists() {
        model_snake = format!("{}s", model_snake);
    }

    let template = format!(
r#"/* ---------------------------------------------------------
 * 📑 LABEL: {class_name} (observers/{file_name})
 * --------------------------------------------------------- */

use crate::app::models::{model_snake}::Model as {model_pascal};
use rustbasic_core::serde_json::Value;

pub trait {class_name} {{
    fn creating(data: &mut Value);
    fn created(model: &{model_pascal});
    fn updating(data: &mut Value);
    fn updated(model: &{model_pascal});
    fn deleting(id: i32);
    fn deleted(id: i32);
}}

pub struct {class_name}Impl;

impl {class_name} for {class_name}Impl {{
    fn creating(_data: &mut Value) {{
        // Lakukan sesuatu sebelum data disimpan ke database (Before Create)
    }}

    fn created(_model: &{model_pascal}) {{
        // Lakukan sesuatu setelah data berhasil disimpan ke database (After Create)
    }}

    fn updating(_data: &mut Value) {{
        // Lakukan sesuatu sebelum data diupdate di database (Before Update)
    }}

    fn updated(_model: &{model_pascal}) {{
        // Lakukan sesuatu setelah data berhasil diupdate di database (After Update)
    }}

    fn deleting(_id: i32) {{
        // Lakukan sesuatu sebelum data dihapus dari database (Before Delete)
    }}

    fn deleted(_id: i32) {{
        // Lakukan sesuatu setelah data berhasil dihapus dari database (After Delete)
    }}
}}
"#, class_name = class_name, file_name = file_name, model_pascal = model_pascal, model_snake = model_snake);

    fs::write(&file_path, template).expect("Gagal membuat file observer");
    println!("{} {}", "✅ Observer dibuat:".green(), file_path.cyan());

    update_observer_mod_rs(&snake_name);
}

pub fn update_observer_mod_rs(mod_name: &str) {
    let mod_path = "src/app/observers/mod.rs";
    let mut content = String::new();
    if let Ok(mut file) = fs::File::open(mod_path) {
        file.read_to_string(&mut content).ok();
    }

    let line = format!("pub mod {}_observer;", mod_name);
    if !content.contains(&line) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(mod_path)
            .expect("Gagal membuka observers/mod.rs");
        writeln!(file, "{}", line).ok();
        println!("{} {}", "📝".blue(), "observers/mod.rs diperbarui.".dimmed());
    }

    // Register observers module in src/app/mod.rs if not already present
    let app_mod_path = "src/app/mod.rs";
    if let Ok(content) = fs::read_to_string(app_mod_path)
        && !content.contains("pub mod observers;") {
        let mut file = OpenOptions::new()
            .append(true)
            .open(app_mod_path)
            .expect("Gagal membuka app/mod.rs");
        writeln!(file, "pub mod observers;").ok();
        println!("{} {}", "📝".blue(), "app/mod.rs diperbarui.".dimmed());
    }
}

pub fn make_service(name: &str) {
    let pascal_name = to_pascal_case(name).replace("Service", "");
    let snake_name = to_snake_case(&pascal_name);
    let class_name = format!("{}Service", pascal_name);
    let file_name = format!("{}_service.rs", snake_name);
    
    // Ensure src/app/services/ directory exists
    let dir_path = "src/app/services";
    fs::create_dir_all(dir_path).expect("Gagal membuat folder services");
    
    let file_path = format!("{}/{}", dir_path, file_name);

    if std::path::Path::new(&file_path).exists() {
        println!("{} {} {}", "⚠️  Service".yellow(), file_path.cyan(), "sudah ada.".yellow());
        return;
    }

    let template = format!(
r#"/* ---------------------------------------------------------
 * 📑 LABEL: {class_name} (services/{file_name})
 * --------------------------------------------------------- */

use rustbasic_core::sql::AnyPool;

pub struct {class_name} {{
    _db: AnyPool,
}}

impl {class_name} {{
    pub fn new(db: AnyPool) -> Self {{
        Self {{ _db: db }}
    }}

    // Tambahkan fungsi logika bisnis Anda di sini
}}
"#, class_name = class_name, file_name = file_name);

    fs::write(&file_path, template).expect("Gagal membuat file service");
    println!("{} {}", "✅ Service dibuat:".green(), file_path.cyan());

    update_service_mod_rs(&snake_name);
}

pub fn update_service_mod_rs(mod_name: &str) {
    let mod_path = "src/app/services/mod.rs";
    let mut content = String::new();
    if let Ok(mut file) = fs::File::open(mod_path) {
        file.read_to_string(&mut content).ok();
    }

    let line = format!("pub mod {}_service;", mod_name);
    if !content.contains(&line) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(mod_path)
            .expect("Gagal membuka services/mod.rs");
        writeln!(file, "{}", line).ok();
        println!("{} {}", "📝".blue(), "services/mod.rs diperbarui.".dimmed());
    }

    // Register services module in src/app/mod.rs if not already present
    let app_mod_path = "src/app/mod.rs";
    if let Ok(content) = fs::read_to_string(app_mod_path)
        && !content.contains("pub mod services;") {
        let mut file = OpenOptions::new()
            .append(true)
            .open(app_mod_path)
            .expect("Gagal membuka app/mod.rs");
        writeln!(file, "pub mod services;").ok();
        println!("{} {}", "📝".blue(), "app/mod.rs diperbarui.".dimmed());
    }
}


