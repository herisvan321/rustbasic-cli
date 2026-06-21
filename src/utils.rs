use rustbasic_core::colored::*;
use std::io::{BufRead, Read, Write};

/// Menampilkan prompt interaktif dan meminta user memilih angka dari min..=max
pub fn prompt_choice(prompt: &str, min: usize, max: usize) -> usize {
    loop {
        print!("{}", prompt);
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok()
            && let Ok(choice) = input.trim().parse::<usize>()
                && choice >= min && choice <= max {
                    return choice;
                }
        println!("⚠️ Pilihan tidak valid, silakan coba lagi.");
    }
}

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

struct CursorGuard;
impl Drop for CursorGuard {
    fn drop(&mut self) {
        print!("\x1B[?25h");
        let _ = std::io::stdout().flush();
    }
}

pub fn parse_cargo_progress(line: &str) -> Option<(usize, usize, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("Building [") {
        return None;
    }
    let close_bracket = trimmed.find(']')?;
    let after_bracket = trimmed[close_bracket + 1..].trim_start();
    
    let colon = after_bracket.find(':')?;
    let fraction_part = after_bracket[..colon].trim();
    
    let slash = fraction_part.find('/')?;
    let current = fraction_part[..slash].parse::<usize>().ok()?;
    let total = fraction_part[slash + 1..].parse::<usize>().ok()?;
    
    let details = after_bracket[colon + 1..].trim().to_string();
    
    Some((current, total, details))
}

pub fn parse_compiling_crate(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.starts_with("Compiling ") || trimmed.starts_with("Checking ") || trimmed.starts_with("Documenting ") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    None
}

pub fn run_cargo_with_progress(mut cmd: std::process::Command) -> std::io::Result<std::process::ExitStatus> {
    // Force cargo term progress configuration
    cmd.arg("--config").arg("term.progress.when=\"always\"");
    cmd.arg("--config").arg("term.progress.width=100");
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.stdin(std::process::Stdio::inherit());
    
    let mut child = cmd.spawn()?;
    
    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();
    
    struct State {
        current: usize,
        total: usize,
        last_crate_action: String,
        last_crate: String,
        active: bool,
    }
    
    let state = std::sync::Arc::new(std::sync::Mutex::new(State {
        current: 0,
        total: 0,
        last_crate_action: "Compiling".to_string(),
        last_crate: String::new(),
        active: false,
    }));
    
    // Hide cursor using CursorGuard
    let _guard = CursorGuard;
    print!("\x1B[?25l");
    let _ = std::io::stdout().flush();
    
    // Helper to draw progress bar
    let draw_progress = |state: &State| {
        if !state.active || state.total == 0 {
            return;
        }
        let width = 30;
        let completed = (state.current * width) / state.total;
        
        let mut bar = String::new();
        for i in 0..width {
            if i < completed {
                // Multicolored gradient blocks: Magenta -> Cyan -> Yellow -> Green
                if i < 8 {
                    bar.push_str(&"█".magenta().to_string());
                } else if i < 16 {
                    bar.push_str(&"█".cyan().to_string());
                } else if i < 24 {
                    bar.push_str(&"█".yellow().to_string());
                } else {
                    bar.push_str(&"█".green().to_string());
                }
            } else {
                bar.push_str(&"░".dimmed().to_string());
            }
        }
        
        let pct = (state.current * 100) / state.total;
        let pct_colored = if pct < 33 {
            format!("{:>3}%", pct).magenta().bold()
        } else if pct < 66 {
            format!("{:>3}%", pct).cyan().bold()
        } else if pct < 90 {
            format!("{:>3}%", pct).yellow().bold()
        } else {
            format!("{:>3}%", pct).green().bold()
        };
        
        let action_label = if state.last_crate_action.starts_with("Check") {
            "Checking".yellow().bold()
        } else if state.last_crate_action.starts_with("Doc") {
            "Documenting".blue().bold()
        } else {
            "Compiling".magenta().bold()
        };
        
        let crate_desc = if state.last_crate.is_empty() {
            "cargo...".white()
        } else {
            format!("{} {}", action_label, state.last_crate.white().bold().italic()).white()
        };
        
        let step_count = format!("({}/{})", state.current, state.total).cyan().dimmed();
        
        print!(
            "\r\x1B[2K  ⚡  [{}] {} {}  {}",
            bar,
            pct_colored,
            step_count,
            crate_desc
        );
        let _ = std::io::stdout().flush();
    };
    
    let state_clone_stdout = std::sync::Arc::clone(&state);
    let stdout_thread = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(child_stdout);
        for line in reader.lines().map_while(Result::ok) {
            let s = state_clone_stdout.lock().unwrap();
            // Clear the progress bar, print stdout line, redraw progress bar
            print!("\r\x1B[2K");
            println!("{}", line);
            draw_progress(&s);
        }
    });
    
    let state_clone_stderr = std::sync::Arc::clone(&state);
    let stderr_thread = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(child_stderr);
        let mut buffer = Vec::new();
        
        loop {
            let mut byte = [0u8; 1];
            match reader.read_exact(&mut byte) {
                Ok(_) => {
                    let b = byte[0];
                    if b == b'\n' || b == b'\r' {
                        if !buffer.is_empty() {
                            let line = String::from_utf8_lossy(&buffer).to_string();
                            buffer.clear();
                            
                            let mut s = state_clone_stderr.lock().unwrap();
                            if let Some((current, total, _details)) = parse_cargo_progress(&line) {
                                s.current = current;
                                s.total = total;
                                s.active = true;
                                draw_progress(&s);
                            } else if let Some((action, crate_name)) = parse_compiling_crate(&line) {
                                s.last_crate_action = action;
                                s.last_crate = crate_name;
                                draw_progress(&s);
                            } else {
                                let trimmed = line.trim();
                                if trimmed.starts_with("Running ") || trimmed.starts_with("Doc-tests ") || trimmed.starts_with("Finished ") {
                                    s.active = false;
                                    print!("\r\x1B[2K");
                                    let _ = std::io::stdout().flush();
                                }
                                if !trimmed.is_empty() && !trimmed.starts_with("Fresh ") && !trimmed.starts_with("Finished ") {
                                    // It's a warning, error, or custom print. Clear bar, print it, redraw
                                    print!("\r\x1B[2K");
                                    eprintln!("{}", line);
                                    draw_progress(&s);
                                }
                            }
                        }
                    } else {
                        buffer.push(b);
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    });
    
    let status = child.wait()?;
    
    // Wait for output threads to finish reading
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();
    
    // Clear progress bar line at the end
    print!("\r\x1B[2K");
    let _ = std::io::stdout().flush();
    
    Ok(status)
}
