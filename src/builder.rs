use std::process::Command;
use std::io::Write;
use std::fs;
use std::path::Path;
use rustbasic_core::colored::*;

fn setup_java_home() {
    if std::env::var("JAVA_HOME").is_err() {
        let mut custom_java_home: Option<String> = None;
        if cfg!(target_os = "macos") {
            let paths = vec![
                "/Applications/Android Studio.app/Contents/jbr/Contents/Home",
                "/Applications/Android Studio.app/Contents/jre/Contents/Home",
                "/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home",
            ];
            for path in &paths {
                if std::path::Path::new(path).exists() {
                    custom_java_home = Some(path.to_string());
                    break;
                }
            }
        } else if cfg!(target_os = "windows") {
            let win_paths = [
                "C:\\Program Files\\Android\\Android Studio\\jbr",
                "C:\\Program Files\\Android\\Android Studio\\jre",
            ];
            for path in &win_paths {
                if std::path::Path::new(path).exists() {
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
                if std::path::Path::new(path).exists() {
                    custom_java_home = Some(path.to_string());
                    break;
                }
            }
        }
        if let Some(jh) = custom_java_home {
            unsafe {
                std::env::set_var("JAVA_HOME", &jh);
            }
        }
    }
}

fn get_cargo_package_name() -> String {
    if let Ok(content) = fs::read_to_string("Cargo.toml") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() >= 2 {
                    return parts[1].trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }
        }
    }
    "rustbasic".to_string()
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
        if !Path::new("native/desktop/src/main.rs").exists() {
            println!("{}", "❌ Error: File native/desktop/src/main.rs tidak ditemukan. Jalankan 'rustbasic-native install' terlebih dahulu.".red().bold());
            return;
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["build", "--bin", "rustbasic-desktop", "--release", "--features", "desktop"]);
        println!("{} {:?}", "🚀 Menjalankan:".blue().bold(), cmd);
        
        let status = cmd.status();
        match status {
            Ok(s) if s.success() => {
                let bin_name = if cfg!(target_os = "windows") {
                    "rustbasic-desktop.exe"
                } else {
                    "rustbasic-desktop"
                };
                let bin_path = Path::new("target/release").join(bin_name);
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

        // Jalankan JNI compilation
        println!(" JNI shared libraries...");
        if !compile_jni_libraries() {
            println!("{}", "❌ Error: Gagal mengompilasi JNI libraries.".red().bold());
            return;
        }

        setup_java_home();
        let jh_val = std::env::var("JAVA_HOME").ok();

        // 1. Generate Keystore if not exists
        let keystore_path = Path::new("native/android/app/release.keystore");
        if !keystore_path.exists() {
            println!("🔑 Menghasilkan developer release keystore baru...");
            let keytool_bin = if let Some(jh) = jh_val.as_ref() {
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
        if gradle_path.exists()
            && let Ok(content) = fs::read_to_string(gradle_path)
                && !content.contains("signingConfigs") {
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

        println!("🔨 Memulai kompilasi APK & AAB menggunakan Gradle...");
        let gradlew_bin = if cfg!(target_os = "windows") { "gradlew.bat" } else { "./gradlew" };
        let mut gradle_cmd = Command::new(gradlew_bin);
        gradle_cmd.args(["assembleRelease", "bundleRelease"]);
        gradle_cmd.current_dir("native/android");

        if let Some(jh) = jh_val.as_ref() {
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

/// Build Docker image — auto-generate Dockerfile jika belum ada
pub fn build_docker(custom_tag: &str) {
    // 1. Cek apakah docker tersedia
    let docker_check = Command::new("docker")
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match docker_check {
        Ok(status) if status.success() => {}
        _ => {
            println!("❌ Docker tidak ditemukan! Pastikan Docker sudah terinstall dan berjalan.");
            println!("   Install Docker: https://docs.docker.com/get-docker/");
            return;
        }
    }

    // 2. Generate Dockerfile jika belum ada
    let dockerfile_path = Path::new("Dockerfile");
    if !dockerfile_path.exists() {
        println!("📝 Membuat Dockerfile...");
        let is_monorepo = Path::new("../rustbasic-core").exists() || Path::new("rustbasic-core").exists();
        let binary_name = get_cargo_package_name();

        let dockerfile_content = if is_monorepo {
            r#"# ============================================================
# RustBasic Docker Build — Multi-stage
# ============================================================

# Stage 1: Builder
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy rustbasic-core (dari konteks workspace root)
COPY rustbasic-core /build/rustbasic-core

# Copy proyek utama rustbasic
COPY rustbasic /build/rustbasic

WORKDIR /build/rustbasic

# Build release binary
RUN cargo build --release --bin rustbasic

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary dari builder stage
COPY --from=builder /build/rustbasic/target/release/rustbasic .

# Copy assets yang diperlukan dari builder stage (lebih aman dan bersih)
COPY --from=builder /build/rustbasic/src/resources/views/ src/resources/views/
COPY --from=builder /build/rustbasic/src/dist/ src/dist/
COPY --from=builder /build/rustbasic/public/ public/
COPY --from=builder /build/rustbasic/database/migrations/ database/migrations/
COPY --from=builder /build/rustbasic/database/seeders/ database/seeders/
COPY --from=builder /build/rustbasic/.env.example .env

# Expose port aplikasi
EXPOSE 4000

CMD ["./rustbasic"]
"#.to_string()
        } else {
            format!(r#"# ============================================================
# RustBasic Docker Build — Multi-stage
# ============================================================

# Stage 1: Builder
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy proyek utama
COPY . .

# Build release binary
RUN cargo build --release --bin {bin_name}

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary dari builder stage
COPY --from=builder /build/target/release/{bin_name} .

# Copy assets yang diperlukan dari builder stage
COPY --from=builder /build/src/resources/views/ src/resources/views/
COPY --from=builder /build/src/dist/ src/dist/
COPY --from=builder /build/public/ public/
COPY --from=builder /build/database/migrations/ database/migrations/
COPY --from=builder /build/database/seeders/ database/seeders/
COPY --from=builder /build/.env.example .env

# Expose port aplikasi
EXPOSE 4000

CMD ["./{bin_name}"]
"#, bin_name = binary_name)
        };

        if let Err(e) = fs::write(dockerfile_path, dockerfile_content) {
            println!("❌ Gagal membuat Dockerfile: {}", e);
            return;
        }
        println!("✅ Dockerfile berhasil dibuat.");
    }

    // 3. Tentukan image tag
    let app_name = std::env::var("BUILD_NAME")
        .or_else(|_| std::env::var("APP_NAME"))
        .unwrap_or_else(|_| get_cargo_package_name())
        .to_lowercase();
    
    let image_tag = if custom_tag.is_empty() {
        format!("{}:latest", app_name)
    } else {
        custom_tag.to_string()
    };

    let core_context = if std::path::Path::new("../rustbasic-core").exists() {
        "core=../rustbasic-core"
    } else {
        "core=."
    };

    // 4. Build Docker image dengan named context 'core' untuk rustbasic-core
    println!("\n🐳 Memulai Docker build...");
    println!("   Image tag: {}", image_tag);
    println!("   Running: docker build --build-context {} -t {} .", core_context, image_tag);

    let mut cmd = Command::new("docker")
        .args(["build", "--build-context", core_context, "-t", &image_tag, "."])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("Gagal menjalankan docker build");

    let status = cmd.wait().expect("Gagal menunggu docker build");

    if status.success() {
        println!("\n✅ Docker build selesai dengan sukses!");
        println!("📦 Image: {}", image_tag);
        println!("\n   Jalankan container (Development/Lokal):");
        println!("   docker run -p 4000:4000 --env-file .env {}", image_tag);
        println!("\n   Jalankan container (Produksi/Server - Auto Restart):");
        println!("   docker run -d -p 80:4000 --restart unless-stopped --env-file .env {}", image_tag);
    } else {
        println!("\n❌ Docker build gagal.");
    }
}

/// Unified interactive build entry point — menampilkan menu untuk memilih target build
pub fn build_interactive(args: &[String]) {
    let mut build_docker_flag = args.iter().any(|arg| arg == "--docker");
    let mut build_desktop = args.iter().any(|arg| arg == "--desktop");
    let mut build_android = args.iter().any(|arg| arg == "--android");
    let mut release_mode = args.iter().any(|arg| arg == "--release" || arg == "-r");
    let mut target_type = String::new(); // apk / aab
    let mut docker_tag = String::new();

    // Parse arguments
    for i in 0..args.len() {
        if args[i] == "--type" && i + 1 < args.len() {
            target_type = args[i+1].to_lowercase();
        }
        if args[i] == "--tag" && i + 1 < args.len() {
            docker_tag = args[i+1].clone();
        }
    }

    if !build_docker_flag && !build_desktop && !build_android {
        let is_native_installed = crate::packages::read_manifest()
            .packages
            .iter()
            .any(|pkg| pkg.name == "rustbasic-native");

        println!("🛠️  RustBasic Build CLI");
        println!("Pilih platform target untuk di-build:");
        println!("  [1] Docker (Container Image)");
        
        let max_choice = if is_native_installed {
            println!("  [2] Desktop Wrapper (Windows, macOS, Linux)");
            println!("  [3] Android Wrapper (APK, AAB)");
            3
        } else {
            1
        };

        let prompt_str = format!("👉 Pilih nomor platform (1-{}): ", max_choice);
        let choice = crate::utils::prompt_choice(&prompt_str, 1, max_choice);
        match choice {
            1 => build_docker_flag = true,
            2 => build_desktop = true,
            3 => build_android = true,
            _ => {}
        }
    }

    if build_docker_flag {
        build_docker(&docker_tag);
    } else if build_desktop {
        let mut target_os = String::new();
        for i in 0..args.len() {
            if args[i] == "--os" && i + 1 < args.len() {
                target_os = args[i+1].clone();
            }
        }

        let mut target_triple = "";
        if target_os.is_empty() {
            println!("\nPilih OS Target Desktop:");
            println!("  [1] Current OS (Sistem saat ini)");
            println!("  [2] macOS Intel (x86_64)");
            println!("  [3] macOS Apple Silicon (aarch64)");
            println!("  [4] Windows MSVC (x86_64-pc-windows-msvc)");
            println!("  [5] Windows GNU (x86_64-pc-windows-gnu - Rekomendasi Cross-compile dari macOS/Linux)");
            println!("  [6] Linux (x86_64-unknown-linux-gnu)");
            let choice = crate::utils::prompt_choice("👉 Pilih nomor target OS (1-6): ", 1, 6);
            match choice {
                2 => target_triple = "x86_64-apple-darwin",
                3 => target_triple = "aarch64-apple-darwin",
                4 => target_triple = "x86_64-pc-windows-msvc",
                5 => target_triple = "x86_64-pc-windows-gnu",
                6 => target_triple = "x86_64-unknown-linux-gnu",
                _ => {}
            }
        } else {
            match target_os.as_str() {
                "macos-intel" | "macos_intel" => target_triple = "x86_64-apple-darwin",
                "macos-silicon" | "macos_silicon" => target_triple = "aarch64-apple-darwin",
                "macos" => {
                    #[cfg(target_arch = "aarch64")]
                    { target_triple = "aarch64-apple-darwin"; }
                    #[cfg(not(target_arch = "aarch64"))]
                    { target_triple = "x86_64-apple-darwin"; }
                }
                "windows" => target_triple = "x86_64-pc-windows-msvc",
                "windows-gnu" => target_triple = "x86_64-pc-windows-gnu",
                "linux" => target_triple = "x86_64-unknown-linux-gnu",
                _ => {
                    println!("⚠️ Warning: Target OS '{}' tidak dikenal, menggunakan default OS saat ini.", target_os);
                }
            }
        }

        if !args.iter().any(|arg| arg == "--release" || arg == "-r" || arg == "--debug" || arg == "-d") {
            println!("\nPilih Mode Build:");
            println!("  [1] Debug (Cepat compile)");
            println!("  [2] Release (Optimasi penuh)");
            let choice = crate::utils::prompt_choice("👉 Pilih nomor mode (1-2): ", 1, 2);
            if choice == 2 {
                release_mode = true;
            }
        }

        println!("\n🖥️  Memulai proses build Desktop...");
        let mut build_args = vec!["build", "--bin", "rustbasic-desktop", "--features", "desktop"];
        if release_mode {
            build_args.push("--release");
        }
        if !target_triple.is_empty() {
            build_args.push("--target");
            build_args.push(target_triple);
            let _ = Command::new("rustup")
                .args(["target", "add", target_triple])
                .status();
        }

        println!("   Running: cargo {}", build_args.join(" "));
        let mut cmd = Command::new("cargo")
            .args(&build_args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("Gagal menjalankan cargo build");
        
        let status = cmd.wait().expect("Gagal menunggu proses cargo build");
        if status.success() {
            println!("\n✅ Build Desktop selesai dengan sukses!");
            let mode_str = if release_mode { "release" } else { "debug" };
            let path_str = if target_triple.is_empty() {
                format!("target/{}/rustbasic-desktop", mode_str)
            } else {
                format!("target/{}/{}/rustbasic-desktop", target_triple, mode_str)
            };
            println!("📂 Output biner: {}", path_str);
        } else {
            println!("\n❌ Build Desktop gagal.");
        }

    } else if build_android {
        let mut is_aab = false;
        if target_type.is_empty() {
            println!("\nPilih Format Output Android:");
            println!("  [1] APK (Android Package - Siap install)");
            println!("  [2] AAB (Android App Bundle - Siap Google Play)");
            let choice = crate::utils::prompt_choice("👉 Pilih format (1-2): ", 1, 2);
            if choice == 2 {
                is_aab = true;
            }
        } else {
            is_aab = target_type == "aab";
        }

        if !args.iter().any(|arg| arg == "--release" || arg == "-r" || arg == "--debug" || arg == "-d") {
            println!("\nPilih Mode Build:");
            println!("  [1] Debug");
            println!("  [2] Release (Produksi)");
            let choice = crate::utils::prompt_choice("👉 Pilih nomor mode (1-2): ", 1, 2);
            if choice == 2 {
                release_mode = true;
            }
        }

        println!("\n🔨 Membangun JNI library untuk Android...");
        if !compile_jni_libraries() {
            println!("❌ Gagal membangun JNI libraries.");
            return;
        }

        // Setup Android home and SDK environments
        let os = std::env::consts::OS;
        let home = std::env::var("HOME").unwrap_or_default();
        let android_home = if let Ok(val) = std::env::var("ANDROID_HOME") {
            val
        } else {
            if os == "macos" {
                format!("{}/Library/Android/sdk", home)
            } else {
                format!("{}/Android/Sdk", home)
            }
        };
        unsafe {
            std::env::set_var("ANDROID_HOME", &android_home);
        }
        
        if std::env::var("JAVA_HOME").is_err() {
            let studio_jbr = if os == "macos" {
                let mut jbr = "/Applications/Android Studio.app/Contents/jbr/Contents/Home".to_string();
                if !Path::new(&jbr).exists() {
                    jbr = "/Applications/Android Studio.app/Contents/jre/Contents/Home".to_string();
                }
                jbr
            } else {
                let mut jbr = "/opt/android-studio/jbr".to_string();
                if !Path::new(&jbr).exists() {
                    jbr = "/usr/local/android-studio/jbr".to_string();
                }
                jbr
            };
            if Path::new(&studio_jbr).exists() {
                unsafe {
                    std::env::set_var("JAVA_HOME", &studio_jbr);
                }
            }
        }

        let local_props = Path::new("native/android/local.properties");
        if !local_props.exists()
            && let Ok(mut file) = fs::File::create(local_props) {
                let _ = writeln!(file, "sdk.dir={}", android_home);
            }

        let gradle_task = match (is_aab, release_mode) {
            (false, false) => "assembleDebug",
            (false, true) => "assembleRelease",
            (true, false) => "bundleDebug",
            (true, true) => "bundleRelease",
        };

        println!("\n🔨 Membangun target Android via Gradle (task: {})...", gradle_task);
        
        let mut build_cmd = if Path::new("native/android/gradlew").exists() {
            let mut cmd = Command::new("./gradlew");
            cmd.arg(gradle_task);
            cmd.current_dir("native/android");
            cmd
        } else {
            let mut cmd = Command::new("gradle");
            cmd.arg(gradle_task);
            cmd.current_dir("native/android");
            cmd
        };

        let spawn_res = build_cmd
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn();

        if let Ok(mut child) = spawn_res {
            let status = child.wait().expect("Gagal menunggu Gradle build");
            if status.success() {
                println!("\n✅ Build Android selesai dengan sukses!");
                let output_dir = if is_aab {
                    let mode_folder = if release_mode { "release" } else { "debug" };
                    format!("native/android/app/build/outputs/bundle/{}", mode_folder)
                } else {
                    let mode_folder = if release_mode { "release" } else { "debug" };
                    format!("native/android/app/build/outputs/apk/{}", mode_folder)
                };
                println!("📂 Folder output: {}", output_dir);
            } else {
                println!("\n❌ Gradle build gagal.");
            }
        } else {
            println!("❌ Gagal mengeksekusi Gradle wrapper. Pastikan Java dan Gradle wrapper terkonfigurasi dengan benar.");
        }
    }
}

/// Jalankan native runner script (Android / Desktop)
pub fn run_native(run_android: bool, run_desktop: bool) {
    if run_android {
        println!("🚀 Memulai RustBasic Android Wrapper...");
        
        let os = std::env::consts::OS;
        let home = std::env::var("HOME").unwrap_or_default();
        let android_home = if let Ok(val) = std::env::var("ANDROID_HOME") {
            val
        } else {
            if os == "macos" {
                format!("{}/Library/Android/sdk", home)
            } else {
                format!("{}/Android/Sdk", home)
            }
        };

        let mut custom_java_home = None;
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

        let adb = get_adb_bin();
        let mut devices = get_adb_devices();
        if devices.is_empty() {
            println!("📱 Perangkat Android atau emulator tidak terdeteksi aktif.");
            let mut emulator_bin = format!("{}/emulator/emulator", android_home);
            if !Path::new(&emulator_bin).exists() {
                let fallback = format!("{}/tools/emulator", android_home);
                if Path::new(&fallback).exists() {
                    emulator_bin = fallback;
                } else {
                    emulator_bin = "emulator".to_string();
                }
            }

            let avd_output = Command::new(&emulator_bin).arg("-list-avds").output();
            if let Ok(avd_out) = avd_output {
                let avds_str = String::from_utf8_lossy(&avd_out.stdout);
                if let Some(avd_name) = avds_str.lines().next() {
                    println!("🚀 Menyalakan emulator AVD: {}...", avd_name);
                    let _ = Command::new(&emulator_bin)
                        .args(["-avd", avd_name])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    
                    println!("⏳ Menunggu emulator menyala dan terdeteksi adb...");
                    let _ = Command::new(&adb).arg("wait-for-device").status();
                    println!("✅ Emulator berhasil aktif!");
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    devices = get_adb_devices();
                }
            }
        }

        let (device_id, device_name) = if devices.len() == 1 {
            let d = devices[0].clone();
            println!("📱 Menggunakan perangkat tunggal: {} ({})", d.1, d.0);
            d
        } else if devices.len() > 1 {
            println!("📱 Terdeteksi beberapa perangkat Android. Silakan pilih target:");
            for (idx, d) in devices.iter().enumerate() {
                println!("  [{}] {} ({})", idx + 1, d.1, d.0);
            }
            let choice = crate::utils::prompt_choice("👉 Pilih nomor perangkat: ", 1, devices.len());
            devices[choice - 1].clone()
        } else {
            println!("❌ Error: Tidak ada perangkat Android terhubung.");
            return;
        };

        // 3. build JNI
        if !compile_jni_libraries() {
            return;
        }

        // 4. local.properties
        let local_props = Path::new("native/android/local.properties");
        if !local_props.exists() {
            if let Ok(mut file) = fs::File::create(local_props) {
                let _ = writeln!(file, "sdk.dir={}", android_home);
            }
        }

        // 5. gradlew assembleDebug
        println!("🔨 Membangun debug APK menggunakan Gradle...");
        let gradlew_bin = if cfg!(target_os = "windows") { "gradlew.bat" } else { "./gradlew" };
        let mut gradle_cmd = Command::new(gradlew_bin);
        gradle_cmd.arg("assembleDebug");
        gradle_cmd.current_dir("native/android");

        if let Some(jh) = &custom_java_home {
            gradle_cmd.env("JAVA_HOME", jh);
        }

        let gradle_status = gradle_cmd.status();
        if gradle_status.is_err() || !gradle_status.unwrap().success() {
            println!("❌ Gradle build assembleDebug gagal.");
            return;
        }

        // 6. adb install
        println!("🔨 Memasang APK ke perangkat {} ({})...", device_name, device_id);
        let install_status = Command::new(&adb)
            .args(["-s", &device_id, "install", "-r", "native/android/app/build/outputs/apk/debug/app-debug.apk"])
            .status();

        if install_status.is_err() || !install_status.unwrap().success() {
            println!("❌ Gagal memasang APK ke device.");
            return;
        }

        // 7. adb reverse
        let vite_port = "5173"; // default
        let reverse_status = Command::new(&adb)
            .args(["-s", &device_id, "reverse", &format!("tcp:{}", vite_port), &format!("tcp:{}", vite_port)])
            .status();
        if reverse_status.is_err() {
            println!("⚠️ Warning: Gagal melakukan adb reverse port {}", vite_port);
        }

        // 8. adb shell am start
        println!("🚀 Membuka aplikasi di perangkat {}...", device_name);
        let _ = Command::new(&adb)
            .args(["-s", &device_id, "logcat", "-c"])
            .status();
        
        let _ = Command::new(&adb)
            .args(["-s", &device_id, "shell", "am", "start", "-n", "com.rustbasic.mobile/com.rustbasic.mobile.MainActivity"])
            .status();

        println!("📋 Menampilkan log realtime dari perangkat {} (Tekan Ctrl+C untuk keluar)...", device_name);
        let mut logcat_cmd = Command::new(&adb);
        logcat_cmd.args(["-s", &device_id, "logcat", "-s", "RustBasicServer"]);
        let mut child = logcat_cmd.spawn().expect("Gagal menjalankan adb logcat");
        let _ = child.wait();
    } else if run_desktop {
        println!("🚀 Memulai RustBasic Desktop Wrapper...");
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--bin", "rustbasic-desktop", "--features", "desktop"]);
        let status = cmd.status();
        match status {
            Ok(s) if s.success() => {}
            _ => {
                println!("❌ Gagal menjalankan Desktop Wrapper.");
            }
        }
    }
}

fn prompt_string(prompt: &str, default: &str) -> String {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        default.to_string()
    }
}

pub fn deploy_interactive() {
    println!("\n{}", "🚀 RustBasic Docker Deploy CLI".magenta().bold());
    println!("{}", "------------------------------".magenta());

    // 1. Konfigurasi Image & Pengiriman
    let image_name = prompt_string("👉 Masukkan Nama/Tag Docker Image (default: rustbasic:latest): ", "rustbasic:latest");
    
    // Tentukan nama container dari env DOCKER_CONTAINER_NAME atau fallback dari nama image
    let container_name = std::env::var("DOCKER_CONTAINER_NAME").unwrap_or_else(|_| {
        let base_name = image_name
            .split(':')
            .next()
            .unwrap_or("rustbasic")
            .split('/')
            .last()
            .unwrap_or("rustbasic")
            .to_lowercase();
        format!("{}-app", base_name)
    });

    // Cek apakah Docker image sudah ada
    let inspect = Command::new("docker")
        .args(["image", "inspect", &image_name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match inspect {
        Ok(status) if status.success() => {}
        _ => {
            println!("{}", format!("⚠️  Peringatan: Image '{}' tidak ditemukan lokal.", image_name).yellow());
            let proceed = prompt_string("👉 Tetap lanjutkan proses ekspor? (y/n) [default: n]: ", "n");
            if proceed.to_lowercase() != "y" {
                println!("❌ Proses dihentikan.");
                return;
            }
        }
    }

    // 2. Ekspor ke tar
    println!("📦 Mengekspor Docker image '{}' ke 'rustbasic.tar'...", image_name);
    let save_status = Command::new("docker")
        .args(["save", "-o", "rustbasic.tar", &image_name])
        .status();

    match save_status {
        Ok(status) if status.success() => {
            println!("{}", "✅ Image berhasil diekspor ke rustbasic.tar.".green());
        }
        _ => {
            println!("{}", "❌ Gagal mengekspor Docker image.".red().bold());
            return;
        }
    }

    // 3. Konfigurasi Pengiriman
    println!("\n{}", "🌐 Konfigurasi Pengiriman ke Server".cyan().bold());
    println!("{}", "-----------------------------------".cyan());
    let ssh_user = prompt_string("👉 Masukkan SSH Username Server (contoh: root) [default: root]: ", "root");
    let ssh_ip = prompt_string("👉 Masukkan IP Address Server: ", "");
    if ssh_ip.is_empty() {
        println!("{}", "❌ IP Address server tidak boleh kosong.".red().bold());
        let _ = fs::remove_file("rustbasic.tar");
        return;
    }
    let ssh_port = prompt_string("👉 Masukkan SSH Port Server (default: 22): ", "22");
    let dest_dir = prompt_string("👉 Masukkan Folder Tujuan di Server (default: ~/app): ", "~/app");
    let server_port = prompt_string("👉 Masukkan Port Server Mapping (contoh: 80:4000) [default: 80:4000]: ", "80:4000");
    let env_file = prompt_string("👉 Masukkan File Env yang akan dikirim (default: .env): ", ".env");

    println!("\n🚀 Menyiapkan folder tujuan di server...");
    let mkdir_status = Command::new("ssh")
        .args([
            "-p", &ssh_port,
            &format!("{}@{}", ssh_user, ssh_ip),
            &format!("mkdir -p {}", dest_dir)
        ])
        .status();

    match mkdir_status {
        Ok(status) if status.success() => {}
        _ => {
            println!("{}", "❌ Gagal terhubung ke server menggunakan SSH.".red().bold());
            let _ = fs::remove_file("rustbasic.tar");
            return;
        }
    }

    println!("🚀 Mengirimkan berkas rustbasic.tar & {} ke server...", env_file);
    let scp_status = Command::new("scp")
        .args([
            "-P", &ssh_port,
            "rustbasic.tar", &env_file,
            &format!("{}@{}:{}", ssh_user, ssh_ip, dest_dir)
        ])
        .status();

    // Hapus file tar lokal
    let _ = fs::remove_file("rustbasic.tar");

    if let Ok(status) = scp_status {
        if !status.success() {
            println!("{}", "❌ Gagal mengirimkan berkas via SCP.".red().bold());
            return;
        }
    } else {
        println!("{}", "❌ Gagal menjalankan SCP.".red().bold());
        return;
    }

    println!("{}", "✅ Pengiriman berkas berhasil!".green());

    // 4. Eksekusi SSH otomatis di server jika disetujui
    let auto_run = prompt_string("\n👉 Apakah Anda ingin langsung menjalankan container di server secara otomatis? (y/n) [default: y]: ", "y");
    if auto_run.to_lowercase() == "y" || auto_run.is_empty() {
        println!("\n🚀 Memuat image di server (docker load)...");
        let load_status = Command::new("ssh")
            .args([
                "-p", &ssh_port,
                &format!("{}@{}", ssh_user, ssh_ip),
                &format!("docker load -i {}/rustbasic.tar", dest_dir)
            ])
            .status();

        match load_status {
            Ok(status) if status.success() => {
                println!("{}", "✅ Image berhasil dimuat di server.".green());
            }
            _ => {
                println!("{}", "❌ Gagal memuat image di server.".red().bold());
                return;
            }
        }

        println!("🚀 Menghentikan & menghapus container lama '{}' jika ada...", container_name);
        let stop_status = Command::new("ssh")
            .args([
                "-p", &ssh_port,
                &format!("{}@{}", ssh_user, ssh_ip),
                &format!("docker stop {} || true && docker rm {} || true", container_name, container_name)
            ])
            .status();

        if let Err(e) = stop_status {
            println!("⚠️ Peringatan saat membersihkan container lama: {}", e);
        }

        println!("🚀 Menjalankan container baru '{}'...", container_name);
        let run_cmd = format!(
            "docker run -d --name {} -p {} --restart unless-stopped --env-file {}/.env {}",
            container_name, server_port, dest_dir, image_name
        );
        let run_status = Command::new("ssh")
            .args([
                "-p", &ssh_port,
                &format!("{}@{}", ssh_user, ssh_ip),
                &run_cmd
            ])
            .status();

        match run_status {
            Ok(status) if status.success() => {
                println!("{}", format!("🎉 Container '{}' berhasil dijalankan di server!", container_name).green().bold());
            }
            _ => {
                println!("{}", "❌ Gagal menjalankan container di server.".red().bold());
                return;
            }
        }

        println!("🚀 Membersihkan file tar di server...");
        let rm_status = Command::new("ssh")
            .args([
                "-p", &ssh_port,
                &format!("{}@{}", ssh_user, ssh_ip),
                &format!("rm {}/rustbasic.tar", dest_dir)
            ])
            .status();

        if let Err(e) = rm_status {
            println!("⚠️ Peringatan saat membersihkan file tar di server: {}", e);
        }

        println!("\n{}", "🎉 Deployment selesai!".green().bold());
        println!("{}", "--------------------------------------------------------".green());
        println!("Untuk melihat log aplikasi di server, jalankan:");
        println!("ssh -p {} {}@{} \"docker logs -f {}\"", ssh_port, ssh_user, ssh_ip, container_name);
        println!("{}", "--------------------------------------------------------".green());
    } else {
        println!("\n{}", "🖥️  Langkah Selanjutnya di Server Anda:".cyan().bold());
        println!("{}", "--------------------------------------------------------".green());
        println!("1. Hubungkan ke server via SSH:");
        println!("   ssh -p {} {}@{}", ssh_port, ssh_user, ssh_ip);
        println!("");
        println!("2. Masuk ke folder tujuan:");
        println!("   cd {}", dest_dir);
        println!("");
        println!("3. Muat (load) image Docker dari berkas tar:");
        println!("   docker load -i rustbasic.tar");
        println!("");
        println!("4. Jalankan container dengan fitur auto-restart:");
        println!("   docker run -d --name {} -p {} --restart unless-stopped --env-file .env {}", container_name, server_port, image_name);
        println!("");
        println!("5. Hapus file tar di server untuk menghemat disk:");
        println!("   rm rustbasic.tar");
        println!("{}", "--------------------------------------------------------".green());
    }
}

fn get_adb_bin() -> String {
    // 1. Cek apakah adb terdaftar di PATH dan bisa dijalankan
    if let Ok(output) = Command::new("adb").arg("devices").output() {
        if output.status.success() {
            return "adb".to_string();
        }
    }

    // 2. Jika tidak ada di PATH, cari di folder SDK default
    let os = std::env::consts::OS;
    let home = std::env::var("HOME").unwrap_or_default();
    let android_home = if let Ok(val) = std::env::var("ANDROID_HOME") {
        val
    } else {
        if os == "macos" {
            format!("{}/Library/Android/sdk", home)
        } else if os == "windows" {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            format!("{}\\Android\\Sdk", local_app_data)
        } else {
            format!("{}/Android/Sdk", home)
        }
    };

    let adb_name = if os == "windows" { "adb.exe" } else { "adb" };
    let path = Path::new(&android_home).join("platform-tools").join(adb_name);
    if path.exists() {
        path.display().to_string()
    } else {
        "adb".to_string() // fallback terakhir jika semua gagal
    }
}

fn get_adb_devices() -> Vec<(String, String)> {
    let adb = get_adb_bin();
    let output = Command::new(&adb).arg("devices").output();
    let mut devices = Vec::new();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("List of devices") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                let device_id = parts[0].to_string();
                let model_out = Command::new(&adb)
                    .args(["-s", &device_id, "shell", "getprop", "ro.product.model"])
                    .output();
                let model = if let Ok(m_out) = model_out {
                    String::from_utf8_lossy(&m_out.stdout).trim().to_string()
                } else {
                    "Unknown Device".to_string()
                };
                devices.push((device_id, model));
            }
        }
    }
    devices
}

fn compile_jni_libraries() -> bool {
    println!("🚀 Building Rust library for Android (Native Rust implementation)...");

    let _ = Command::new("rustup")
        .args(["target", "add", "aarch64-linux-android", "armv7-linux-androideabi", "x86_64-linux-android"])
        .status();

    let home = std::env::var("HOME").unwrap_or_default();
    let android_ndk_home = if let Ok(val) = std::env::var("ANDROID_NDK_HOME") {
        val
    } else {
        let mac_ndk = format!("{}/Library/Android/sdk/ndk", home);
        if Path::new(&mac_ndk).exists() {
            if let Ok(entries) = fs::read_dir(&mac_ndk) {
                let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
                paths.sort();
                if let Some(highest) = paths.last() {
                    highest.display().to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    };

    if android_ndk_home.is_empty() {
        println!("❌ Error: ANDROID_NDK_HOME is not set. Please set ANDROID_NDK_HOME.");
        return false;
    }

    println!("Using NDK: {}", android_ndk_home);

    let os = std::env::consts::OS;
    let toolchain_sub = if os == "macos" { "darwin-x86_64" } else { "linux-x86_64" };
    let toolchain_bin_path = Path::new(&android_ndk_home)
        .join("toolchains/llvm/prebuilt")
        .join(toolchain_sub)
        .join("bin");

    if !toolchain_bin_path.exists() {
        println!("❌ Error: Toolchain bin path not found: {}", toolchain_bin_path.display());
        return false;
    }

    let sqlite_version = "3450100";
    let sqlite_dir = format!("target/sqlite-amalgamation-{}", sqlite_version);
    if !Path::new(&sqlite_dir).exists() {
        println!("📥 Downloading SQLite source amalgamation...");
        fs::create_dir_all("target").ok();
        
        let zip_path = "target/sqlite.zip";
        let sqlite_url = format!("https://www.sqlite.org/2024/sqlite-amalgamation-{}.zip", sqlite_version);
        
        let curl_status = Command::new("curl")
            .args(["-sSLo", zip_path, &sqlite_url])
            .status();
        
        if curl_status.is_err() || !curl_status.unwrap().success() {
            println!("❌ Gagal men-download SQLite source.");
            return false;
        }

        let unzip_status = Command::new("unzip")
            .args(["-q", zip_path, "-d", "target/"])
            .status();

        let _ = fs::remove_file(zip_path);

        if unzip_status.is_err() || !unzip_status.unwrap().success() {
            println!("❌ Gagal mengekstrak SQLite source.");
            return false;
        }
    }

    let targets = vec![
        ("aarch64-linux-android", "arm64-v8a", "aarch64-linux-android21-clang"),
        ("armv7-linux-androideabi", "armeabi-v7a", "armv7a-linux-androideabi21-clang"),
        ("x86_64-linux-android", "x86_64", "x86_64-linux-android21-clang"),
    ];

    let jnilibs_dir = "native/android/app/src/main/jniLibs";

    for (target, arch, clang_name) in targets {
        println!("🔨 Preparing SQLite static library for {}...", target);
        
        let clang_path = toolchain_bin_path.join(clang_name);
        let ar_path = toolchain_bin_path.join("llvm-ar");

        if !clang_path.exists() {
            println!("❌ Error: Compiler not found: {}", clang_path.display());
            return false;
        }

        let sqlite_out = format!("target/{}/sqlite", target);
        fs::create_dir_all(&sqlite_out).ok();

        let libsqlite3_a = format!("{}/libsqlite3.a", sqlite_out);
        if !Path::new(&libsqlite3_a).exists() {
            println!("   Compiling SQLite static lib for {}...", target);
            let sqlite3_o = format!("{}/sqlite3.o", sqlite_out);
            let sqlite3_c = format!("{}/sqlite3.c", sqlite_dir);
            
            let compile_status = Command::new(&clang_path)
                .args(["-O2", "-c", &sqlite3_c, "-o", &sqlite3_o])
                .status();

            if compile_status.is_err() || !compile_status.unwrap().success() {
                println!("❌ Gagal mengompilasi sqlite3.o");
                return false;
            }

            let archive_status = Command::new(&ar_path)
                .args(["rcs", &libsqlite3_a, &sqlite3_o])
                .status();

            if archive_status.is_err() || !archive_status.unwrap().success() {
                println!("❌ Gagal mengarsip libsqlite3.a");
                return false;
            }
        }

        println!("🔨 Compiling Rust library for {}...", target);
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.args(["build", "--target", target, "--release"]);

        let clang_path_str = clang_path.display().to_string();
        let ar_path_str = ar_path.display().to_string();

        let target_upper = target.replace("-", "_").to_uppercase();
        let linker_env = format!("CARGO_TARGET_{}_LINKER", target_upper);
        let cc_env = format!("CC_{}", target.replace("-", "_"));
        let ar_env = format!("AR_{}", target.replace("-", "_"));

        cargo_cmd.env(&linker_env, &clang_path_str);
        cargo_cmd.env(&cc_env, &clang_path_str);
        cargo_cmd.env(&ar_env, &ar_path_str);

        let cargo_status = cargo_cmd.status();
        if cargo_status.is_err() || !cargo_status.unwrap().success() {
            println!("❌ Gagal mengompilasi library Rust untuk target {}", target);
            return false;
        }

        let dest_dir = format!("{}/{}", jnilibs_dir, arch);
        fs::create_dir_all(&dest_dir).ok();

        let src_so = format!("target/{}/release/librustbasic.so", target);
        let dest_so = format!("{}/librustbasic_mobile.so", dest_dir);

        if let Err(e) = fs::copy(&src_so, &dest_so) {
            println!("❌ Gagal menyalin {}: {}", src_so, e);
            return false;
        }
    }

    println!("✅ Android JNI libraries built successfully!");
    true
}
