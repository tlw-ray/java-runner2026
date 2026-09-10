use crate::config::{parse_launch_args, AppConfig};
use crate::i18n;
use crate::model::JvmInstallation;
use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn prompt_and_run(
    installations: &[JvmInstallation],
    config: &AppConfig,
    work_dir: &Path,
    preselected: Option<usize>,
    pause_before_exit: bool,
) -> Result<()> {
    if installations.is_empty() {
        bail!("{}", i18n::messages().no_jvm);
    }

    let index = resolve_jvm_index(installations, config, preselected)?;

    let jvm = &installations[index];
    let args = parse_launch_args(&config.launch_args);
    run_java_in_terminal(jvm, &args, work_dir, pause_before_exit)
}

fn resolve_jvm_index(
    installations: &[JvmInstallation],
    config: &AppConfig,
    preselected: Option<usize>,
) -> Result<usize> {
    if let Some(n) = preselected {
        return validate_selection(installations, n);
    }

    if let Some(raw_hint) = config.java_home.as_deref().filter(|s| !s.trim().is_empty()) {
        let hint = raw_hint.to_ascii_lowercase();
        if let Some((idx, jvm)) = installations.iter().enumerate().find(|(_, jvm)| {
            jvm.home
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains(&hint)
        }) {
            let mut stderr = io::stderr().lock();
            writeln!(
                stderr,
                "{}",
                i18n::format_java_home_matched(idx + 1, &jvm.home.display().to_string())
            )
            .ok();
            return Ok(idx);
        }
        bail!("{}", i18n::format_java_home_not_found(raw_hint.trim()));
    }

    if installations.len() == 1 {
        let jvm = &installations[0];
        let mut stderr = io::stderr().lock();
        writeln!(
            stderr,
            "{}",
            i18n::format_single_jvm_auto(&jvm.home.display().to_string())
        )
        .ok();
        return Ok(0);
    }

    interactive_select(installations)
}

fn validate_selection(installations: &[JvmInstallation], one_based: usize) -> Result<usize> {
    if one_based == 0 || one_based > installations.len() {
        bail!(
            "{}",
            i18n::format_invalid_selection(one_based, installations.len())
        );
    }
    Ok(one_based - 1)
}

fn interactive_select(installations: &[JvmInstallation]) -> Result<usize> {
    let m = i18n::messages();
    let mut stderr = io::stderr().lock();
    writeln!(stderr).ok();
    writeln!(stderr, "{}", m.prompt_select_jvm).ok();

    loop {
        write!(stderr, "> ").ok();
        stderr.flush().ok();

        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .context(m.read_input_failed)?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            writeln!(
                stderr,
                "{}",
                i18n::format_prompt_empty_input(installations.len())
            )
            .ok();
            continue;
        }

        match trimmed.parse::<usize>() {
            Ok(n) => match validate_selection(installations, n) {
                Ok(index) => return Ok(index),
                Err(err) => {
                    writeln!(stderr, "{err}").ok();
                }
            },
            Err(_) => {
                writeln!(stderr, "{}", m.prompt_invalid_number).ok();
            }
        }
    }
}

pub fn run_java_in_terminal(
    jvm: &JvmInstallation,
    args: &[String],
    work_dir: &Path,
    pause_before_exit: bool,
) -> Result<()> {
    let m = i18n::messages();
    let mut stderr = io::stderr().lock();
    writeln!(stderr).ok();
    write!(stderr, "{}: {}", m.using_jvm, jvm.home.display()).ok();
    if let Some(info) = &jvm.version_info {
        write!(stderr, " ({})", info.version).ok();
    }
    writeln!(stderr).ok();

    let cmd_display = format_command_display(&jvm.java_exe, args);
    writeln!(stderr, "{}: {cmd_display}", m.running_command).ok();
    writeln!(stderr).ok();
    stderr.flush().ok();

    crate::util::ensure_utf8_console();

    let mut command = Command::new(&jvm.java_exe);
    command
        .args(args)
        .env("JAVA_HOME", &jvm.home)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    command.current_dir(work_dir);
    crate::util::apply_java_utf8_env(&mut command);

    if let Ok(path) = std::env::var("PATH") {
        let bin = jvm.home.join("bin");
        let new_path = format!("{};{}", bin.display(), path);
        command.env("PATH", new_path);
    }

    let java_path = jvm.java_exe.display().to_string();
    let status = command
        .status()
        .with_context(|| i18n::format_launch_failed(&java_path))?;

    if let Some(code) = status.code() {
        if pause_before_exit {
            crate::util::wait_for_any_key();
        }
        std::process::exit(code);
    }

    bail!("{}", m.java_terminated_abnormally);
}

fn format_command_display(java_exe: &Path, args: &[String]) -> String {
    let mut parts = vec![quote_if_needed(&java_exe.display().to_string())];
    parts.extend(args.iter().map(|a| quote_if_needed(a)));
    parts.join(" ")
}

fn quote_if_needed(s: &str) -> String {
    if s.contains(' ') {
        format!("\"{s}\"")
    } else {
        s.to_string()
    }
}
