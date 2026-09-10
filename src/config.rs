use anyhow::{Context, Result};
use serde::Deserialize;
use serde::de::{self, Visitor};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::util::exe_dir;

const CONFIG_FILE_NAME: &str = "java-runner2026.toml";
const CONFIG_DIR_PATH: &str = "config/java-runner2026.toml";

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// 路径片段，匹配 JVM 安装目录后自动选用（跳过交互选择）。
    #[serde(default)]
    pub java_home: Option<String>,
    /// 版本信息关键字；若设置，仅列举 `java -version` 输出中包含该关键字的 JVM。
    #[serde(default)]
    pub version_key: Option<String>,
    #[serde(default = "default_launch_args", deserialize_with = "deserialize_launch_args")]
    pub launch_args: Vec<String>,
}

/// TOML 中 `launch_args` 可为字符串或字符串数组。
fn deserialize_launch_args<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct LaunchArgsVisitor;

    impl<'de> Visitor<'de> for LaunchArgsVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or an array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_string()])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut out = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                out.push(item);
            }
            Ok(out)
        }
    }

    deserializer.deserialize_any(LaunchArgsVisitor)
}

fn default_launch_args() -> Vec<String> {
    vec!["-version".to_string()]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            java_home: None,
            version_key: None,
            launch_args: default_launch_args(),
        }
    }
}

pub fn load_config(explicit: Option<&Path>) -> Result<(AppConfig, PathBuf)> {
    if let Some(path) = explicit {
        let config = load_config_file(path).with_context(|| {
            crate::i18n::format_read_config_failed(&path.display().to_string())
        })?;
        return Ok((config, work_dir_for_config(path)));
    }

    for candidate in default_config_paths() {
        if candidate.is_file() {
            let config = load_config_file(&candidate).with_context(|| {
                crate::i18n::format_read_config_failed(&candidate.display().to_string())
            })?;
            return Ok((config, work_dir_for_config(&candidate)));
        }
    }

    let work_dir = exe_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((AppConfig::default(), work_dir))
}

/// 配置文件所在应用根目录（config/java-runner2026.toml → 上级目录）。
pub fn work_dir_for_config(config_path: &Path) -> PathBuf {
    let parent = config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    if parent.file_name().and_then(|s| s.to_str()) == Some("config") {
        parent
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(parent)
            .to_path_buf()
    } else {
        parent.to_path_buf()
    }
}

fn default_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(dir) = exe_dir() {
        paths.push(dir.join(CONFIG_FILE_NAME));
        paths.push(dir.join(CONFIG_DIR_PATH));
    }
    paths.push(PathBuf::from(CONFIG_FILE_NAME));
    paths.push(PathBuf::from(CONFIG_DIR_PATH));
    paths
}

fn load_config_file(path: &Path) -> Result<AppConfig> {
    let text = std::fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&text)?;
    Ok(config)
}

/// 将配置中的 launch_args 解析为 java.exe 参数列表。
/// 支持 TOML 数组，也兼容单个字符串 "launch_args = \"-version\""
pub fn parse_launch_args(args: &[String]) -> Vec<String> {
    if args.len() == 1 {
        return shlex_split(&args[0]);
    }
    args.to_vec()
}

fn shlex_split(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        out.push(current);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_quoted_args() {
        assert_eq!(
            shlex_split(r#"-cp "C:\lib\app.jar" com.example.Main"#),
            vec![
                "-cp".to_string(),
                r"C:\lib\app.jar".to_string(),
                "com.example.Main".to_string(),
            ]
        );
    }

    #[test]
    fn loads_string_launch_args() {
        let config: AppConfig = toml::from_str(r#"launch_args = "-version""#).unwrap();
        assert_eq!(config.launch_args, vec!["-version".to_string()]);
        assert_eq!(parse_launch_args(&config.launch_args), vec!["-version".to_string()]);
    }

    #[test]
    fn loads_array_launch_args() {
        let config: AppConfig =
            toml::from_str(r#"launch_args = ["-jar", "app.jar"]"#).unwrap();
        assert_eq!(
            config.launch_args,
            vec!["-jar".to_string(), "app.jar".to_string()]
        );
    }

    #[test]
    fn resolves_work_dir_from_config_path() {
        let dir = work_dir_for_config(Path::new("D:/app/config/java-runner2026.toml"));
        assert_eq!(dir, PathBuf::from("D:/app"));
        let dir = work_dir_for_config(Path::new("D:/app/java-runner2026.toml"));
        assert_eq!(dir, PathBuf::from("D:/app"));
    }

    #[test]
    fn loads_literal_string_with_colon_paths() {
        let config: AppConfig = toml::from_str(
            r#"launch_args = '-Xloggc:logs/gc.log -XX:HeapDumpPath=logs/oom -jar magic-boot-1.0.jar'"#,
        )
        .unwrap();
        assert_eq!(
            parse_launch_args(&config.launch_args),
            vec![
                "-Xloggc:logs/gc.log".to_string(),
                "-XX:HeapDumpPath=logs/oom".to_string(),
                "-jar".to_string(),
                "magic-boot-1.0.jar".to_string(),
            ]
        );
    }

    #[test]
    fn loads_quoted_string_launch_args() {
        let config: AppConfig =
            toml::from_str(r#"launch_args = "-cp \"target/app.jar\" com.example.Main""#)
                .unwrap();
        assert_eq!(
            parse_launch_args(&config.launch_args),
            vec![
                "-cp".to_string(),
                "target/app.jar".to_string(),
                "com.example.Main".to_string(),
            ]
        );
    }

    #[test]
    fn loads_version_key() {
        let config: AppConfig = toml::from_str(r#"version_key = "1.8""#).unwrap();
        assert_eq!(config.version_key.as_deref(), Some("1.8"));
    }
}
