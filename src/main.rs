mod cli;
mod config;
mod i18n;
mod launcher;
mod model;
mod scan;
mod util;

use cli::Args;
use config::load_config;
use launcher::prompt_and_run;
use model::{filter_by_version_key, JvmInstallation};
use scan::{scan_all, ScanStats};
use std::io::{self, Write};

fn should_pause(args: &Args) -> bool {
    !args.json && !args.no_pause
}

fn main() {
    util::ensure_utf8_console();
    i18n::init();
    let args = cli::parse_args();
    if let Err(err) = run(&args) {
        let m = i18n::messages();
        eprintln!("{}: {err}", m.error_prefix);
        if should_pause(&args) {
            util::wait_for_any_key();
        }
        std::process::exit(1);
    }
}

fn run(args: &Args) -> anyhow::Result<()> {
    let m = i18n::messages();

    eprintln!("{}", m.scanning);
    let (all_installations, stats) = scan_all();
    let (config, work_dir) = load_config(args.config.as_deref())?;

    let version_key = config
        .version_key
        .as_deref()
        .filter(|s| !s.trim().is_empty());
    let installations = if let Some(key) = version_key {
        filter_by_version_key(&all_installations, key)
    } else {
        all_installations
    };

    if args.json {
        print_json(&installations, &stats, args.verbose)?;
        return Ok(());
    }

    if installations.is_empty() {
        if stats.unique_installations > 0 {
            if let Some(key) = version_key {
                println!("{}", i18n::format_version_key_not_found(key.trim()));
            } else {
                println!("{}", m.no_jvm);
            }
        } else {
            println!("{}", m.no_jvm);
        }
        if should_pause(args) {
            util::wait_for_any_key();
        }
        return Ok(());
    }

    if let Some(key) = version_key {
        eprintln!(
            "{}",
            i18n::format_version_key_filtered(key.trim(), installations.len())
        );
    }

    print_human(&installations, &stats, args.verbose);

    if args.scan_only {
        if should_pause(args) {
            util::wait_for_any_key();
        }
        return Ok(());
    }

    prompt_and_run(
        &installations,
        &config,
        &work_dir,
        args.select,
        should_pause(args),
    )?;

    Ok(())
}

fn print_human(installations: &[JvmInstallation], stats: &ScanStats, verbose: bool) {
    let m = i18n::messages();
    let mut out = io::stdout().lock();

    writeln!(out).ok();
    debug_assert!(!installations.is_empty());

    writeln!(
        out,
        "{}",
        i18n::format_jvm_summary(installations.len(), stats.raw_candidates, stats.elapsed)
    )
    .ok();
    writeln!(out).ok();

    for (idx, jvm) in installations.iter().enumerate() {
        writeln!(out, "[{}] {}", idx + 1, jvm.home.display()).ok();
        writeln!(out, "    java.exe: {}", jvm.java_exe.display()).ok();

        match &jvm.version_info {
            Some(info) => {
                writeln!(out, "    {}: {}", m.label_version, info.version).ok();
                if let Some(product) = &info.product {
                    writeln!(out, "    {}: {product}", m.label_product).ok();
                }
                if let Some(runtime) = &info.runtime {
                    writeln!(out, "    {}: {runtime}", m.label_runtime).ok();
                }
                if let Some(vm) = &info.vm {
                    writeln!(out, "    {}: {vm}", m.label_vm).ok();
                }
            }
            None => {
                writeln!(out, "    {}: {}", m.label_version, m.version_probe_failed).ok();
            }
        }

        if verbose {
            writeln!(out, "    {}:", m.discovery_sources).ok();
            for (source, detail) in &jvm.sources {
                writeln!(out, "      - {} ({detail})", source.label()).ok();
            }
        }

        writeln!(out).ok();
    }
}

fn print_json(
    installations: &[JvmInstallation],
    stats: &ScanStats,
    verbose: bool,
) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct Output<'a> {
        count: usize,
        raw_candidates: usize,
        elapsed_ms: u128,
        jvms: Vec<JvmOutput<'a>>,
    }

    #[derive(serde::Serialize)]
    struct JvmOutput<'a> {
        home: String,
        java_exe: String,
        version: Option<String>,
        product: Option<String>,
        runtime: Option<String>,
        vm: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sources: Option<&'a [(model::DiscoverySource, String)]>,
    }

    let jvms: Vec<JvmOutput> = installations
        .iter()
        .map(|jvm| JvmOutput {
            home: jvm.home.display().to_string(),
            java_exe: jvm.java_exe.display().to_string(),
            version: jvm.version_info.as_ref().map(|v| v.version.clone()),
            product: jvm.version_info.as_ref().and_then(|v| v.product.clone()),
            runtime: jvm.version_info.as_ref().and_then(|v| v.runtime.clone()),
            vm: jvm.version_info.as_ref().and_then(|v| v.vm.clone()),
            sources: if verbose {
                Some(&jvm.sources)
            } else {
                None
            },
        })
        .collect();

    let output = Output {
        count: stats.unique_installations,
        raw_candidates: stats.raw_candidates,
        elapsed_ms: stats.elapsed.as_millis(),
        jvms,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
