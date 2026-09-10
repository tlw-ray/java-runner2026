use crate::model::{DiscoverySource, JvmCandidate};
use crate::util::{expand_env, is_java_related_env_name, resolve_jvm_from_env_value};
use std::path::PathBuf;

pub fn scan_environment() -> Vec<JvmCandidate> {
    let mut out = Vec::new();

    // 扫描所有名称含 JAVA / JDK / JRE 的环境变量
    for (name, raw) in std::env::vars() {
        if !is_java_related_env_name(&name) {
            continue;
        }
        if let Some((home, java_exe)) = resolve_jvm_from_env_value(&raw) {
            out.push(JvmCandidate {
                home,
                java_exe,
                source: DiscoverySource::EnvironmentVariable,
                detail: crate::i18n::detail_env_var(&name),
            });
        }
    }

    // PATH 单独处理：查找其中的 java.exe
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let expanded = expand_env(dir);
            let java_exe = PathBuf::from(&expanded).join("java.exe");
            if java_exe.is_file() {
                if let Some(home) = java_exe.parent().and_then(|b| b.parent()) {
                    out.push(JvmCandidate {
                        home: home.to_path_buf(),
                        java_exe: Some(java_exe),
                        source: DiscoverySource::PathVariable,
                        detail: crate::i18n::detail_path_dir(&expanded),
                    });
                }
            }
        }
    }

    out
}
