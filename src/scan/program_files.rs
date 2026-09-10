use crate::model::{DiscoverySource, JvmCandidate};
use crate::util::program_files_dirs;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const JAVA_DIR_NAMES: &[&str] = &[
    "Java",
    "Eclipse Adoptium",
    "Eclipse Foundation",
    "Microsoft",
    "Amazon Corretto",
    "Azul",
    "Zulu",
    "BellSoft",
    "Liberica",
    "AdoptOpenJDK",
    "GraalVM",
    "Semeru",
    "OpenJDK",
    "jdk",
    "jre",
];

pub fn scan_program_files() -> Vec<JvmCandidate> {
    let roots: Vec<PathBuf> = program_files_dirs();
    roots
        .par_iter()
        .flat_map(|root| scan_root(root))
        .collect()
}

fn scan_root(root: &Path) -> Vec<JvmCandidate> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return out;
    }

    for name in JAVA_DIR_NAMES {
        let candidate = root.join(name);
        if candidate.is_dir() {
            collect_java_homes(&candidate, &format!("{}\\{}", root.display(), name), &mut out);
        }
    }

    // 直接在 Program Files 下扫描 jdk-* / jre-* 目录
    if let Ok(read_dir) = std::fs::read_dir(root) {
        for entry in read_dir.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if file_name.starts_with("jdk")
                || file_name.starts_with("jre")
                || file_name.contains("java")
                || file_name.contains("corretto")
                || file_name.contains("graalvm")
            {
                push_if_java_home(
                    &path,
                    &crate::i18n::detail_program_files_direct(&path.display().to_string()),
                    &mut out,
                );
            }
        }
    }

    out
}

fn collect_java_homes(base: &Path, detail_prefix: &str, out: &mut Vec<JvmCandidate>) {
    push_if_java_home(base, detail_prefix, out);

    for entry in WalkDir::new(base)
        .max_depth(2)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_dir() && path != base {
            let name = path
                .strip_prefix(base)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| path.display().to_string());
            push_if_java_home(path, &format!("{detail_prefix}\\{name}"), out);
        }
    }
}

fn push_if_java_home(path: &Path, detail: &str, out: &mut Vec<JvmCandidate>) {
    let java_exe = path.join("bin").join("java.exe");
    if java_exe.is_file() {
        out.push(JvmCandidate {
            home: path.to_path_buf(),
            java_exe: Some(java_exe),
            source: DiscoverySource::ProgramFiles,
            detail: detail.to_string(),
        });
    }
}
