use crate::model::{is_plausible_java_home, normalize_home, resolve_java_exe, JavaVersionInfo};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 当前可执行文件所在目录（用于定位配置与 -jar 工作目录）。
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

/// 将 Windows 控制台切换为 UTF-8（代码页 65001），避免 Java 中文日志乱码。
pub fn ensure_utf8_console() {
    #[cfg(windows)]
    {
        const CP_UTF8: u32 = 65001;
        unsafe {
            extern "system" {
                fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
                fn SetConsoleCP(wCodePageID: u32) -> i32;
            }
            let _ = SetConsoleOutputCP(CP_UTF8);
            let _ = SetConsoleCP(CP_UTF8);
        }
    }
}

const JAVA_UTF8_TOOL_OPTIONS: &str = "-Dfile.encoding=UTF-8 -Dsun.jnu.encoding=UTF-8 \
    -Dsun.stdout.encoding=UTF-8 -Dsun.stderr.encoding=UTF-8 -Dconsole.encoding=UTF-8 \
    -Dlogging.charset.console=UTF-8";

/// 为 Java 子进程补充 UTF-8 相关环境变量（不覆盖已有 JAVA_TOOL_OPTIONS）。
pub fn apply_java_utf8_env(command: &mut Command) {
    let merged = match std::env::var("JAVA_TOOL_OPTIONS") {
        Ok(existing) if !existing.is_empty() => format!("{existing} {JAVA_UTF8_TOOL_OPTIONS}"),
        _ => JAVA_UTF8_TOOL_OPTIONS.to_string(),
    };
    command.env("JAVA_TOOL_OPTIONS", merged);
}

pub fn expand_env(value: &str) -> String {
    if value.contains('%') {
        expand_windows_env(value)
    } else {
        value.to_string()
    }
}

fn expand_windows_env(value: &str) -> String {
    let mut out = value.to_string();
    for _ in 0..8 {
        if !out.contains('%') {
            break;
        }
        let Some(start) = out.find('%') else { break };
        let Some(end) = out[start + 1..].find('%') else { break };
        let end = start + 1 + end;
        let name = &out[start + 1..end];
        let replacement = std::env::var(name).unwrap_or_default();
        out.replace_range(start..=end, &replacement);
    }
    out
}

pub fn parse_command_line(command: &str) -> Option<PathBuf> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }

    let token = if trimmed.starts_with('"') {
        trimmed
            .strip_prefix('"')
            .and_then(|s| s.split('"').next())
            .map(str::trim)
    } else {
        trimmed.split_whitespace().next()
    }?;

    let expanded = expand_env(token);
    let path = PathBuf::from(expanded);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

pub fn java_home_from_java_exe(java_exe: &Path) -> Option<PathBuf> {
    java_exe
        .parent()
        .and_then(|bin| bin.parent())
        .map(Path::to_path_buf)
}

/// 环境变量名是否包含 JAVA / JDK / JRE（不区分大小写）
pub fn is_java_related_env_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.contains("JAVA") || upper.contains("JDK") || upper.contains("JRE")
}

/// 分析环境变量值是否指向 JVM 安装目录，返回 (JAVA_HOME, java.exe)
pub fn resolve_jvm_from_env_value(value: &str) -> Option<(PathBuf, Option<PathBuf>)> {
    let expanded = expand_env(value.trim());
    if expanded.is_empty() {
        return None;
    }

    let path = PathBuf::from(&expanded);

    // 值直接指向 java.exe / javaw.exe
    if path.is_file() {
        let file_name = path.file_name()?.to_string_lossy();
        if file_name.eq_ignore_ascii_case("java.exe") || file_name.eq_ignore_ascii_case("javaw.exe")
        {
            let home = normalize_home(&path);
            return Some((home, Some(path)));
        }
    }

    // 值指向 bin 目录
    if path.ends_with("bin") {
        let home = normalize_home(&path);
        if is_plausible_java_home(&home) {
            return Some((home.clone(), resolve_java_exe(&home)));
        }
    }

    // 值直接是 JAVA_HOME
    if is_plausible_java_home(&path) {
        let home = normalize_home(&path);
        return Some((home.clone(), resolve_java_exe(&home)));
    }

    None
}

/// 执行 java -version 并解析完整版本信息
pub fn probe_java_version(java_exe: &Path) -> Option<JavaVersionInfo> {
    let output = Command::new(java_exe).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if !stderr.trim().is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };
    parse_java_version_output(&text)
}

fn parse_java_version_output(text: &str) -> Option<JavaVersionInfo> {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return None;
    }

    let first = lines[0];
    let version = extract_quoted_version(first)
        .or_else(|| extract_token_version(first))
        .unwrap_or_else(|| "unknown".to_string());

    let product = extract_product(first);
    let runtime = lines.get(1).map(|s| s.to_string());
    let vm = lines.get(2).map(|s| s.to_string());

    Some(JavaVersionInfo {
        version,
        product,
        runtime,
        vm,
    })
}

fn extract_product(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let idx = lower.find("version")?;
    let product = line[..idx].trim();
    if product.is_empty() {
        None
    } else {
        Some(product.to_string())
    }
}

fn extract_quoted_version(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_token_version(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let idx = lower.find("version")? + "version".len();
    let rest = line[idx..].trim();
    let token = rest.split_whitespace().next()?;
    Some(token.trim_matches('"').to_string())
}

pub fn program_files_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for key in ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"] {
        if let Ok(p) = std::env::var(key) {
            dirs.push(PathBuf::from(p));
        }
    }
    dirs.push(PathBuf::from(r"C:\Program Files"));
    dirs.push(PathBuf::from(r"C:\Program Files (x86)"));
    crate::model::unique_paths(dirs)
}

/// 等待用户按键后再退出（避免双击 exe 时窗口闪退）
pub fn wait_for_any_key() {
    eprintln!();
    eprint!("{}", crate::i18n::messages().press_any_key);
    let _ = std::io::stderr().flush();

    #[cfg(windows)]
    {
        unsafe {
            extern "C" {
                fn _getch() -> i32;
            }
            _getch();
        }
    }

    #[cfg(not(windows))]
    {
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }

    eprintln!();
}
