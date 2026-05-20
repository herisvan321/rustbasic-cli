pub fn to_snake_case(s: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}
pub fn to_pascal_case(s: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize_next = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            pascal.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            pascal.push(ch);
        }
    }
    pascal
}

pub fn open_browser(url: &str) {
    let _ = match std::env::consts::OS {
        "macos" => std::process::Command::new("open").arg(url).spawn(),
        "windows" => std::process::Command::new("cmd").args(["/C", "start", url]).spawn(),
        _ => std::process::Command::new("xdg-open").arg(url).spawn(),
    };
}

pub fn wait_and_open(url: String) {
    let addr = url.replace("http://", "").replace("https://", "");
    let addr = addr.split('/').next().unwrap_or(&addr).to_string();
    
    std::thread::spawn(move || {
        // Coba hubungkan ke port selama 60 detik (120 * 500ms)
        for _ in 0..120 {
            if std::net::TcpStream::connect(&addr).is_ok() {
                open_browser(&url);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

pub fn remove_dir_all_recursive(path: &std::path::Path) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                remove_dir_all_recursive(&path)?;
            } else {
                #[cfg(windows)]
                {
                    let mut perms = std::fs::metadata(&path)?.permissions();
                    if perms.readonly() {
                        perms.set_readonly(false);
                        std::fs::set_permissions(&path, perms)?;
                    }
                }
                std::fs::remove_file(&path)?;
            }
        }
        #[cfg(windows)]
        {
            let mut perms = std::fs::metadata(path)?.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                std::fs::set_permissions(path, perms)?;
            }
        }
        std::fs::remove_dir(path)?;
    } else if path.exists() {
        #[cfg(windows)]
        {
            let mut perms = std::fs::metadata(path)?.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                std::fs::set_permissions(path, perms)?;
            }
        }
        std::fs::remove_file(path)?;
    }
    Ok(())
}
