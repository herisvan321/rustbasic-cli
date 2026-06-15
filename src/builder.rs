use std::process::Command;
use std::io::{self, Write};
use std::fs;
use std::path::{Path, PathBuf};
use rustbasic_core::colored::*;

/// Fungsi rekursif untuk menyalin seluruh isi folder
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Fungsi untuk memperbarui variabel APP_DEBUG di file .env
fn update_env_app_debug(is_release: bool) {
    let env_path = Path::new(".env");
    if !env_path.exists() {
        return;
    }
    if let Ok(content) = fs::read_to_string(env_path) {
        let target_value = if is_release { "false" } else { "true" };
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut found = false;
        for line in &mut lines {
            if line.trim_start().starts_with("APP_DEBUG=") {
                *line = format!("APP_DEBUG={}", target_value);
                found = true;
                break;
            }
        }
        if !found {
            lines.push(format!("APP_DEBUG={}", target_value));
        }
        let new_content = lines.join("\n") + "\n";
        if let Err(e) = fs::write(env_path, new_content) {
            println!("{} {}", "⚠️  Gagal memperbarui file .env:".yellow(), e);
        } else {
            println!("{} {}{}", "📝".green(), "APP_DEBUG diatur ke ".dimmed(), target_value.cyan());
        }
    }
}

/// Fungsi untuk mendapatkan port aplikasi dari file .env
fn get_app_port() -> u16 {
    let env_path = Path::new(".env");
    if env_path.exists() {
        if let Ok(content) = fs::read_to_string(env_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("APP_PORT=") {
                    if let Some(port_str) = trimmed.split('=').nth(1) {
                        if let Ok(port) = port_str.trim().parse::<u16>() {
                            return port;
                        }
                    }
                }
            }
        }
    }
    4000
}

pub fn build_project() {
    println!("\n{}", "🚀 RustBasic Build Manager".magenta().bold());
    println!("{}", "--------------------------".magenta());
    
    // 1. Pilih Target
    println!("{}", "--- Pilih Target OS ---".cyan().bold());
    println!("1) Native (Sesuai OS Anda)");
    println!("2) Windows x86_64 (x86_64-pc-windows-msvc)");
    println!("3) Linux x86_64 GNU (x86_64-unknown-linux-gnu)");
    println!("4) Linux x86_64 MUSL (x86_64-unknown-linux-musl)");
    println!("5) Linux ARM64 GNU (aarch64-unknown-linux-gnu)");
    println!("6) Linux ARM64 MUSL (aarch64-unknown-linux-musl)");
    println!("7) macOS ARM64 (aarch64-apple-darwin)");
    println!("8) macOS Intel (x86_64-apple-darwin)");
    println!("9) Batal");
    print!("\n{}", "Masukkan pilihan target (1-9): ".bold());
    io::stdout().flush().unwrap();

    let mut target_choice = String::new();
    io::stdin().read_line(&mut target_choice).ok();
    let target_choice = target_choice.trim();

    if target_choice == "9" {
        println!("{}", "👋 Build dibatalkan.".yellow());
        return;
    }

    let target = match target_choice {
        "2" => Some("x86_64-pc-windows-msvc"),
        "3" => Some("x86_64-unknown-linux-gnu"),
        "4" => Some("x86_64-unknown-linux-musl"),
        "5" => Some("aarch64-unknown-linux-gnu"),
        "6" => Some("aarch64-unknown-linux-musl"),
        "7" => Some("aarch64-apple-darwin"),
        "8" => Some("x86_64-apple-darwin"),
        _ => None, // Native
    };

    // 2. Pilih Mode
    println!("\n{}", "--- Pilih Mode Build ---".cyan().bold());
    println!("1) Development");
    println!("2) Production (Release)");
    print!("\n{}", "Masukkan pilihan mode (1-2): ".bold());
    io::stdout().flush().unwrap();

    let mut mode_choice = String::new();
    io::stdin().read_line(&mut mode_choice).ok();
    let is_release = mode_choice.trim() == "2";

    // 3. Update File .env (APP_DEBUG)
    println!("\n{}", "🔧 Menyiapkan konfigurasi .env...".blue());
    update_env_app_debug(is_release);

    // 4. Jalankan npm run build untuk Frontend
    if Path::new("package.json").exists() {
        println!("\n{}", "📦 Memulai kompilasi aset frontend (npm run build)...".blue());
        let npm_cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
        let status = Command::new(npm_cmd)
            .args(["run", "build"])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("{}", "✅ Kompilasi frontend berhasil!".green().bold());
            }
            Ok(s) => {
                println!("{} {}", "❌ Error: npm run build keluar dengan kode:".red().bold(), s);
                println!("{}", "⚠️  Proses build dihentikan karena kompilasi frontend gagal.".yellow());
                return;
            }
            Err(e) => {
                println!("{} {}", "❌ Error: Gagal mengeksekusi 'npm'. Pastikan npm terinstal.".red().bold(), e);
                return;
            }
        }
    }

    // 5. Eksekusi Build Rust
    println!("\n{}", "🛠️  Menyiapkan build Rust...".blue());

    let has_zigbuild = Command::new("cargo")
        .arg("zigbuild")
        .arg("--version")
        .output()
        .is_ok();

    let mut use_zigbuild = false;
    if has_zigbuild && target.is_some() {
        println!("\n{}", "--- Pilih Compiler untuk Kompilasi Silang ---".cyan().bold());
        println!("1) Cargo Build Standard (Membutuhkan target toolchain terpasang)");
        println!("2) Cargo Zigbuild (Lebih mudah untuk kompilasi silang)");
        print!("\n{}", "Masukkan pilihan compiler (1-2, default 2): ".bold());
        io::stdout().flush().unwrap();

        let mut compiler_choice = String::new();
        io::stdin().read_line(&mut compiler_choice).ok();
        let choice = compiler_choice.trim();
        if choice == "1" {
            use_zigbuild = false;
        } else {
            use_zigbuild = true;
        }
    }

    let mut cmd = if use_zigbuild {
        println!("{}", "✨ Menggunakan cargo-zigbuild untuk kompilasi silang...".green().italic());
        let mut c = Command::new("cargo");
        c.arg("zigbuild");
        c
    } else {
        if let Some(t) = target {
            println!("{} {} {}", "📦 Menambahkan target".blue(), t.yellow(), "via rustup...".blue());
            Command::new("rustup")
                .args(["target", "add", t])
                .status()
                .ok();
        }
        let mut c = Command::new("cargo");
        c.arg("build");
        c
    };

    if is_release {
        cmd.arg("--release");
    }

    if let Some(t) = target {
        cmd.arg("--target").arg(t);
    }

    println!("{} {:?}", "🚀 Menjalankan:".blue().bold(), cmd);
    let status = crate::utils::run_cargo_with_progress(cmd).expect("Gagal menjalankan perintah build");

    if status.success() {
        println!("\n{}", "✅ Build Rust berhasil!".green().bold());
        
        // 6. Siapkan folder deploy dan salin aset
        println!("\n{}", "📂 Menyiapkan folder deploy...".cyan().bold());
        
        let deploy_dir = Path::new("deploy");
        if deploy_dir.exists() {
            println!("{}", "🧹 Membersihkan folder deploy lama...".dimmed());
            let _ = crate::utils::remove_dir_all_recursive(deploy_dir);
        }
        
        if let Err(e) = fs::create_dir_all(deploy_dir) {
            println!("{} {}", "❌ Gagal membuat folder deploy:".red().bold(), e);
            return;
        }

        // Salin folder database jika ada
        if Path::new("database").exists() {
            print!("   {} Menyalin folder database... ", "📦".blue());
            io::stdout().flush().unwrap();
            if copy_dir_all("database", "deploy/database").is_ok() {
                println!("{}", "selesai".green());
            } else {
                println!("{}", "gagal".red());
            }
        }

        // Salin folder dist jika ada
        let mut dist_copied = false;
        if Path::new("src/dist").exists() {
            print!("   {} Menyalin folder src/dist... ", "📦".blue());
            io::stdout().flush().unwrap();
            if copy_dir_all("src/dist", "deploy/dist").is_ok() {
                println!("{}", "selesai".green());
                dist_copied = true;
            } else {
                println!("{}", "gagal".red());
            }
        }

        if !dist_copied && Path::new("dist").exists() {
            print!("   {} Menyalin folder dist... ", "📦".blue());
            io::stdout().flush().unwrap();
            if copy_dir_all("dist", "deploy/dist").is_ok() {
                println!("{}", "selesai".green());
            } else {
                println!("{}", "gagal".red());
            }
        }

        // Salin folder storage jika ada
        if Path::new("storage").exists() {
            print!("   {} Menyalin folder storage... ", "📦".blue());
            io::stdout().flush().unwrap();
            if copy_dir_all("storage", "deploy/storage").is_ok() {
                println!("{}", "selesai".green());
            } else {
                println!("{}", "gagal".red());
            }
        }

        // Salin file .env jika ada
        if Path::new(".env").exists() {
            print!("   {} Menyalin file .env... ", "📄".blue());
            io::stdout().flush().unwrap();
            if fs::copy(".env", "deploy/.env").is_ok() {
                println!("{}", "selesai".green());
            } else {
                println!("{}", "gagal".red());
            }
        }

        // Generate konfigurasi server deployment (.htaccess & nginx.conf)
        let app_port = get_app_port();

        // 1. Generate .htaccess untuk Apache / Shared Hosting
        let htaccess_content = format!(
            r#"<IfModule mod_rewrite.c>
    RewriteEngine On

    # 1. Jika meminta file statis yang ada di folder dist, sajikan langsung dari dist/
    RewriteCond %{{DOCUMENT_ROOT}}/dist/$1 -f
    RewriteRule ^(.*)$ dist/$1 [L]

    # 2. Jika bukan file statis nyata, teruskan ke binary RustBasic yang berjalan di port {}
    RewriteCond %{{REQUEST_FILENAME}} !-f
    RewriteCond %{{REQUEST_FILENAME}} !-d
    RewriteRule ^(.*)$ http://127.0.0.1:{}/$1 [P,L]
    
    RewriteRule ^$ http://127.0.0.1:{}/ [P,L]
</IfModule>
"#,
            app_port, app_port, app_port
        );
        if fs::write("deploy/.htaccess", htaccess_content).is_ok() {
            println!("   {} Menghasilkan file konfigurasi deploy/.htaccess... selesai", "📄".blue());
        }

        // 2. Generate nginx.conf untuk VPS / Nginx Reverse Proxy
        let nginx_content = format!(
            r#"server {{
    listen 80;
    server_name domainanda.com;

    # Root diarahkan ke folder deploy
    root /path/ke/folder/deploy;

    # Coba sajikan file statis dari folder dist jika ada
    location / {{
        try_files /dist$uri @rust_backend;
    }}

    # Teruskan request dinamis ke binary RustBasic yang berjalan di port {}
    location @rust_backend {{
        proxy_pass http://127.0.0.1:{};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}
"#,
            app_port, app_port
        );
        if fs::write("deploy/nginx.conf", nginx_content).is_ok() {
            println!("   {} Menghasilkan file konfigurasi deploy/nginx.conf... selesai", "📄".blue());
        }

        // Salin file binary hasil kompilasi
        let source_binary_filename = if let Some(t) = target {
            if t.contains("windows") {
                "rustbasic.exe"
            } else {
                "rustbasic"
            }
        } else if cfg!(target_os = "windows") {
            "rustbasic.exe"
        } else {
            "rustbasic"
        };

        // Membaca nama kustom untuk hasil build dari .env
        let build_name = std::env::var("BUILD_NAME")
            .unwrap_or_else(|_| "rustbasic".to_string());

        let dest_binary_filename = if let Some(t) = target {
            if t.contains("windows") {
                format!("{}.exe", build_name)
            } else {
                build_name.clone()
            }
        } else if cfg!(target_os = "windows") {
            format!("{}.exe", build_name)
        } else {
            build_name.clone()
        };

        let mode_dir = if is_release { "release" } else { "debug" };
        let binary_path = if let Some(t) = target {
            PathBuf::from("target")
                .join(t)
                .join(mode_dir)
                .join(source_binary_filename)
        } else {
            PathBuf::from("target")
                .join(mode_dir)
                .join(source_binary_filename)
        };

        if binary_path.exists() {
            let dest_binary_path = deploy_dir.join(&dest_binary_filename);
            print!("   {} Menyalin binary ke deploy/{}... ", "🚀".blue(), dest_binary_filename.cyan());
            io::stdout().flush().unwrap();
            if fs::copy(&binary_path, &dest_binary_path).is_ok() {
                println!("{}", "selesai".green());
                println!("\n🎉 {}", "Proses deployment berhasil disiapkan di folder 'deploy'!".green().bold());
            } else {
                println!("{}", "gagal".red());
            }
        } else {
            println!("\n{}", format!("❌ File binary tidak ditemukan di: {}", binary_path.display()).red().bold());
        }
    } else {
        println!("\n{}", "❌ Build Rust gagal.".red().bold());
        println!("{}", "💡 Penyebab: Linker untuk target tersebut tidak ditemukan di sistem Anda.".yellow());
        
        if target_choice == "2" {
            println!("\n{}", "🔧 Cara memperbaiki untuk Windows:".cyan());
            println!("   Jalankan: {}", "brew install mingw-w64".white().on_black());
        } else if target_choice == "3" {
            println!("\n{}", "🔧 Cara memperbaiki untuk Linux:".cyan());
            println!("   Jalankan: {}", "brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu".white().on_black());
        }
        
        println!("\n{}", "Atau gunakan 'cargo-zigbuild' untuk kompilasi silang yang lebih mudah:".cyan());
        println!("1. brew install zig");
        println!("2. cargo install cargo-zigbuild");
        println!("3. Gunakan '{}'", "cargo zigbuild --target <target>".white().on_black());
    }
}
pub fn build_native_project(run_android: bool, run_desktop: bool) {
    println!("\n{}", "🚀 RustBasic Native Build Manager".magenta().bold());
    println!("{}", "---------------------------------".magenta());

    // 1. Jalankan npm run build untuk Frontend
    if Path::new("package.json").exists() {
        println!("\n{}", "📦 Memulai kompilasi aset frontend (npm run build)...".blue());
        let npm_cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
        let status = Command::new(npm_cmd)
            .args(["run", "build"])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("{}", "✅ Kompilasi frontend berhasil!".green().bold());
            }
            Ok(s) => {
                println!("{} {}", "❌ Error: npm run build keluar dengan kode:".red().bold(), s);
                println!("{}", "⚠️  Proses build dihentikan karena kompilasi frontend gagal.".yellow());
                return;
            }
            Err(e) => {
                println!("{} {}", "❌ Error: Gagal mengeksekusi 'npm'. Pastikan npm terinstal.".red().bold(), e);
                return;
            }
        }
    }

    if run_desktop {
        println!("\n{}", "🛠️  Menyiapkan build Desktop Wrapper...".blue());
        if !Path::new("native/desktop/Cargo.toml").exists() {
            println!("{}", "❌ Error: Folder native/desktop tidak ditemukan. Jalankan 'rustbasic-native install' terlebih dahulu.".red().bold());
            return;
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["build", "--manifest-path", "native/desktop/Cargo.toml", "--release"]);
        println!("{} {:?}", "🚀 Menjalankan:".blue().bold(), cmd);
        
        let status = cmd.status();
        match status {
            Ok(s) if s.success() => {
                let bin_name = if cfg!(target_os = "windows") {
                    "rustbasic-native-desktop.exe"
                } else {
                    "rustbasic-native-desktop"
                };
                let bin_path = Path::new("native/desktop/target/release").join(bin_name);
                println!("\n🎉 {}", "Build Desktop Wrapper berhasil!".green().bold());
                println!("🚀 Hasil executable berada di: {}", bin_path.display().to_string().cyan().bold());
            }
            _ => {
                println!("\n❌ {}", "Build Desktop Wrapper gagal.".red().bold());
            }
        }
    }
    if run_android {
        println!("\n{}", "🛠️  Menyiapkan build Android Wrapper...".blue());
        if !Path::new("native/android/build.gradle").exists() {
            println!("{}", "❌ Error: Folder native/android tidak ditemukan. Jalankan 'rustbasic-native install' terlebih dahulu.".red().bold());
            return;
        }

        // Jalankan build-android.sh untuk compile JNI release
        println!(" JNI shared libraries...");
        let sh_cmd = if cfg!(target_os = "windows") { "sh" } else { "bash" };
        let status = Command::new(sh_cmd)
            .arg("./native/build-android.sh")
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("{}", "✅ Kompilasi JNI shared libraries berhasil!".green().bold());
            }
            _ => {
                println!("{}", "❌ Error: Gagal mengompilasi JNI libraries.".red().bold());
                return;
            }
        }

        // Tentukan JAVA_HOME jika belum diatur
        let has_java_home = std::env::var("JAVA_HOME").is_ok();
        let mut custom_java_home = None;
        if !has_java_home {
            // Coba deteksi Android Studio JDK di berbagai platform
            if cfg!(target_os = "macos") {
                let mac_studio_jdk = "/Applications/Android Studio.app/Contents/jbr/Contents/Home";
                if Path::new(mac_studio_jdk).exists() {
                    custom_java_home = Some(mac_studio_jdk.to_string());
                }
            } else if cfg!(target_os = "windows") {
                let win_paths = [
                    "C:\\Program Files\\Android\\Android Studio\\jbr",
                    "C:\\Program Files\\Android\\Android Studio\\jre",
                ];
                for path in &win_paths {
                    if Path::new(path).exists() {
                        custom_java_home = Some(path.to_string());
                        break;
                    }
                }
            } else {
                // Linux & other Unix-like OS
                let unix_paths = [
                    "/opt/android-studio/jbr",
                    "/opt/android-studio/jre",
                    "/snap/android-studio/current/jbr",
                    "/snap/android-studio/current/jre",
                    "/usr/local/android-studio/jbr",
                    "/usr/local/android-studio/jre",
                    "/usr/lib/jvm/default-java",
                ];
                for path in &unix_paths {
                    if Path::new(path).exists() {
                        custom_java_home = Some(path.to_string());
                        break;
                    }
                }
            }
        }

        // 1. Generate Keystore if not exists
        let keystore_path = Path::new("native/android/app/release.keystore");
        if !keystore_path.exists() {
            println!("🔑 Menghasilkan developer release keystore baru...");
            let keytool_bin = if let Some(jh) = custom_java_home.as_ref() {
                let jh_bin = Path::new(jh).join("bin/keytool");
                if jh_bin.exists() {
                    jh_bin.display().to_string()
                } else {
                    "keytool".to_string()
                }
            } else {
                "keytool".to_string()
            };

            let mut keytool_cmd = Command::new(keytool_bin);
            keytool_cmd.args([
                "-genkeypair",
                "-v",
                "-keystore",
                "native/android/app/release.keystore",
                "-alias",
                "rustbasic",
                "-keyalg",
                "RSA",
                "-keysize",
                "2048",
                "-validity",
                "10000",
                "-storepass",
                "rustbasic",
                "-keypass",
                "rustbasic",
                "-dname",
                "CN=RustBasic Developer, O=RustBasic, C=ID"
            ]);
            let _ = keytool_cmd.status();
        }

        // 2. Inject signingConfigs into build.gradle if not already present
        let gradle_path = Path::new("native/android/app/build.gradle");
        if gradle_path.exists() {
            if let Ok(content) = fs::read_to_string(gradle_path) {
                if !content.contains("signingConfigs") {
                    println!("📝 Menyematkan konfigurasi tanda tangan (signingConfigs) ke build.gradle...");
                    let updated_content = content
                        .replace(
                            "buildTypes {",
                            "signingConfigs {\n        release {\n            storeFile file(\"release.keystore\")\n            storePassword \"rustbasic\"\n            keyAlias \"rustbasic\"\n            keyPassword \"rustbasic\"\n        }\n    }\n\n    buildTypes {"
                        )
                        .replace(
                            "buildTypes {\n        release {\n            minifyEnabled",
                            "buildTypes {\n        release {\n            signingConfig signingConfigs.release\n            minifyEnabled"
                        )
                        .replace(
                            "buildTypes {\r\n        release {\r\n            minifyEnabled",
                            "buildTypes {\r\n        release {\r\n            signingConfig signingConfigs.release\r\n            minifyEnabled"
                        );
                    let _ = fs::write(gradle_path, updated_content);
                }
            }
        }

        println!("🔨 Memulai kompilasi APK & AAB menggunakan Gradle...");
        let gradlew_bin = if cfg!(target_os = "windows") { "gradlew.bat" } else { "./gradlew" };
        let mut gradle_cmd = Command::new(gradlew_bin);
        gradle_cmd.args(["assembleRelease", "bundleRelease"]);
        gradle_cmd.current_dir("native/android");

        if let Some(jh) = custom_java_home.as_ref() {
            gradle_cmd.env("JAVA_HOME", jh);
        }

        let status = gradle_cmd.status();
        match status {
            Ok(s) if s.success() => {
                println!("\n🎉 {}", "Build Android Wrapper berhasil!".green().bold());
                println!("📦 Hasil output:");
                let apk_signed = "native/android/app/build/outputs/apk/release/app-release.apk";
                let final_apk = if Path::new(apk_signed).exists() {
                    apk_signed
                } else {
                    "native/android/app/build/outputs/apk/release/app-release-unsigned.apk"
                };
                println!("   - APK: {}", final_apk.cyan().bold());
                println!("   - AAB: {}", "native/android/app/build/outputs/bundle/release/app-release.aab".cyan().bold());
            }
            _ => {
                println!("\n❌ {}", "Build Android Wrapper gagal.".red().bold());
            }
        }
    }
}

