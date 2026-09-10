use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum DiscoverySource {
    Registry,
    EnvironmentVariable,
    ProgramFiles,
    FileAssociation,
    PathVariable,
    WhereCommand,
    CommonPath,
}

impl DiscoverySource {
    pub fn label(&self) -> &'static str {
        crate::i18n::discovery_source_label(*self)
    }
}

#[derive(Debug, Clone)]
pub struct JvmCandidate {
    pub home: PathBuf,
    pub java_exe: Option<PathBuf>,
    pub source: DiscoverySource,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JavaVersionInfo {
    pub version: String,
    pub product: Option<String>,
    pub runtime: Option<String>,
    pub vm: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JvmInstallation {
    pub home: PathBuf,
    pub java_exe: PathBuf,
    pub version_info: Option<JavaVersionInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<(DiscoverySource, String)>,
}

pub fn normalize_home(path: &Path) -> PathBuf {
    let mut p = path.to_path_buf();
    if p.ends_with("bin") {
        if let Some(parent) = p.parent() {
            p = parent.to_path_buf();
        }
    }
    if p.ends_with("bin/java.exe") || p.ends_with("bin\\java.exe") {
        if let Some(parent) = p.parent().and_then(|b| b.parent()) {
            p = parent.to_path_buf();
        }
    }
    PathBuf::from(
        p.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_string(),
    )
}

pub fn dedupe_candidates(candidates: Vec<JvmCandidate>) -> Vec<JvmInstallation> {
    use std::collections::HashMap;

    let mut map: HashMap<String, JvmInstallation> = HashMap::new();

    for c in candidates {
        let home = normalize_home(&c.home);
        if !is_plausible_java_home(&home) {
            continue;
        }

        let java_exe = c
            .java_exe
            .clone()
            .or_else(|| resolve_java_exe(&home))
            .filter(|p| p.is_file());

        let Some(java_exe) = java_exe else {
            continue;
        };

        let key = path_key(&home);
        map.entry(key)
            .and_modify(|inst| {
                if !inst
                    .sources
                    .iter()
                    .any(|(s, d)| *s == c.source && d == &c.detail)
                {
                    inst.sources.push((c.source, c.detail.clone()));
                }
            })
            .or_insert_with(|| JvmInstallation {
                home,
                java_exe,
                version_info: None,
                sources: vec![(c.source, c.detail)],
            });
    }

    let mut result: Vec<_> = map.into_values().collect();
    result.sort_by(|a, b| a.home.cmp(&b.home));
    result
}

pub fn resolve_java_exe(home: &Path) -> Option<PathBuf> {
    let exe = home.join("bin").join("java.exe");
    if exe.is_file() {
        Some(exe)
    } else {
        None
    }
}

pub fn is_plausible_java_home(home: &Path) -> bool {
    home.join("bin").join("java.exe").is_file()
        || home.join("bin").join("javaw.exe").is_file()
}

pub fn path_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

/// 检查 JVM 版本信息是否包含指定关键字（不区分大小写）。
pub fn matches_version_key(jvm: &JvmInstallation, key: &str) -> bool {
    let key = key.trim();
    if key.is_empty() {
        return true;
    }
    let key = key.to_ascii_lowercase();
    let Some(info) = &jvm.version_info else {
        return false;
    };
    [Some(info.version.as_str()), info.product.as_deref(), info.runtime.as_deref(), info.vm.as_deref()]
        .into_iter()
        .flatten()
        .any(|text| text.to_ascii_lowercase().contains(&key))
}

pub fn filter_by_version_key(installations: &[JvmInstallation], key: &str) -> Vec<JvmInstallation> {
    installations
        .iter()
        .filter(|jvm| matches_version_key(jvm, key))
        .cloned()
        .collect()
}

pub fn unique_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in paths {
        let key = path_key(&p);
        if seen.insert(key) {
            out.push(p);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_jvm(version: &str) -> JvmInstallation {
        JvmInstallation {
            home: PathBuf::from(r"C:\Java\jdk"),
            java_exe: PathBuf::from(r"C:\Java\jdk\bin\java.exe"),
            version_info: Some(JavaVersionInfo {
                version: version.to_string(),
                product: Some("OpenJDK".to_string()),
                runtime: None,
                vm: None,
            }),
            sources: Vec::new(),
        }
    }

    #[test]
    fn version_key_matches_version_field_case_insensitive() {
        let jvm = sample_jvm("1.8.0_271");
        assert!(matches_version_key(&jvm, "1.8"));
        assert!(matches_version_key(&jvm, "OPENJDK"));
        assert!(!matches_version_key(&jvm, "17"));
    }

    #[test]
    fn version_key_excludes_jvm_without_version_info() {
        let mut jvm = sample_jvm("17");
        jvm.version_info = None;
        assert!(!matches_version_key(&jvm, "17"));
    }

    #[test]
    fn filter_by_version_key_returns_matching_installations() {
        let installations = vec![
            sample_jvm("1.8.0_271"),
            sample_jvm("17.0.9"),
        ];
        let filtered = filter_by_version_key(&installations, "1.8");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].version_info.as_ref().unwrap().version, "1.8.0_271");
    }
}
