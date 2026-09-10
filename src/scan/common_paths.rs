use crate::model::{DiscoverySource, JvmCandidate};
use std::path::{Path, PathBuf};

const COMMON_PATHS: &[&str] = &[
    r"C:\Java",
    r"C:\jdk",
    r"C:\jre",
    r"C:\openjdk",
    r"C:\Program Files\Java",
    r"C:\Program Files\Eclipse Adoptium",
    r"C:\Program Files\Amazon Corretto",
    r"C:\Program Files\Microsoft",
    r"C:\Program Files\Zulu",
    r"C:\Program Files\BellSoft",
    r"C:\Program Files\GraalVM",
    r"D:\Java",
    r"D:\jdk",
    r"D:\Program Files\Java",
];

pub fn scan_common_paths() -> Vec<JvmCandidate> {
    let mut out = Vec::new();

    for base in COMMON_PATHS {
        let path = PathBuf::from(base);
        if !path.is_dir() {
            continue;
        }

        push_if_valid(&path, &crate::i18n::detail_common_path(base), &mut out);

        if let Ok(read_dir) = std::fs::read_dir(&path) {
            for entry in read_dir.filter_map(Result::ok) {
                let child = entry.path();
                if child.is_dir() {
                    push_if_valid(
                        &child,
                        &crate::i18n::detail_common_path(&child.display().to_string()),
                        &mut out,
                    );
                }
            }
        }
    }

    out
}

fn push_if_valid(path: &Path, detail: &str, out: &mut Vec<JvmCandidate>) {
    let java_exe = path.join("bin").join("java.exe");
    if java_exe.is_file() {
        out.push(JvmCandidate {
            home: path.to_path_buf(),
            java_exe: Some(java_exe),
            source: DiscoverySource::CommonPath,
            detail: detail.to_string(),
        });
    }
}
