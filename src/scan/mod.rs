mod associations;
mod common_paths;
mod environment;
mod program_files;
mod registry;

use crate::model::{dedupe_candidates, JvmCandidate, JvmInstallation};
use crate::util::probe_java_version;
use associations::{scan_file_associations, scan_where_java};
use common_paths::scan_common_paths;
use environment::scan_environment;
use program_files::scan_program_files;
use registry::scan_registry;
use rayon::prelude::*;
use std::time::Instant;

pub fn scan_all() -> (Vec<JvmInstallation>, ScanStats) {
    let started = Instant::now();

    let chunks: Vec<Vec<JvmCandidate>> = [
        scan_registry,
        scan_environment,
        scan_program_files,
        scan_file_associations,
        scan_where_java,
        scan_common_paths,
    ]
    .par_iter()
    .map(|scan| scan())
    .collect();

    let candidates: Vec<JvmCandidate> = chunks.into_iter().flatten().collect();
    let raw_count = candidates.len();
    let mut installations = dedupe_candidates(candidates);

    // 对每个发现的 JVM 执行 java -version 获取真实版本信息
    installations.par_iter_mut().for_each(|inst| {
        inst.version_info = probe_java_version(&inst.java_exe);
    });

    let elapsed = started.elapsed();
    let stats = ScanStats {
        raw_candidates: raw_count,
        unique_installations: installations.len(),
        elapsed,
    };

    (installations, stats)
}

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub raw_candidates: usize,
    pub unique_installations: usize,
    pub elapsed: std::time::Duration,
}
