use crate::model::{DiscoverySource, JvmCandidate};
use crate::util::{java_home_from_java_exe, parse_command_line};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

pub fn scan_file_associations() -> Vec<JvmCandidate> {
    let mut out = Vec::new();

    scan_progid_command(r".jar", &mut out);
    scan_progid_command(r".class", &mut out);
    scan_applications_java(&mut out);

    out
}

fn scan_progid_command(ext: &str, out: &mut Vec<JvmCandidate>) {
    let ext_key_path = ext;
    if let Ok(ext_key) = RegKey::predef(HKEY_CLASSES_ROOT).open_subkey(ext_key_path) {
        if let Ok(progid) = ext_key.get_value::<String, _>("") {
            let command_path = format!(r"{progid}\shell\open\command");
            read_association_command(&command_path, out, &format!("HKCR\\{command_path}"));
        }

        for name in ext_key.enum_values().filter_map(Result::ok) {
            if name.0.starts_with("Progid") || name.0.eq_ignore_ascii_case("OpenWithProgids") {
                continue;
            }
        }

        if let Ok(open_with) = ext_key.open_subkey("OpenWithProgids") {
            for progid in open_with.enum_keys().filter_map(Result::ok) {
                let command_path = format!(r"{progid}\shell\open\command");
                read_association_command(
                    &command_path,
                    out,
                    &format!("HKCR\\{ext}\\OpenWithProgids\\{progid}"),
                );
            }
        }
    }
}

fn scan_applications_java(out: &mut Vec<JvmCandidate>) {
    let keys = [
        r"Applications\java.exe\shell\open\command",
        r"Applications\javaw.exe\shell\open\command",
    ];
    for subkey in keys {
        read_association_command(subkey, out, &format!("HKCR\\{subkey}"));
    }
}

fn read_association_command(subkey: &str, out: &mut Vec<JvmCandidate>, detail: &str) {
    if let Ok(key) = RegKey::predef(HKEY_CLASSES_ROOT).open_subkey(subkey) {
        if let Ok(command) = key.get_value::<String, _>("") {
            if let Some(java_exe) = parse_command_line(&command) {
                if let Some(home) = java_home_from_java_exe(&java_exe) {
                    out.push(JvmCandidate {
                        home,
                        java_exe: Some(java_exe),
                        source: DiscoverySource::FileAssociation,
                        detail: detail.to_string(),
                    });
                }
            }
        }
    }
}

pub fn scan_where_java() -> Vec<JvmCandidate> {
    let mut out = Vec::new();
    let output = std::process::Command::new("where")
        .arg("java")
        .output();

    let Ok(output) = output else {
        return out;
    };

    if !output.status.success() {
        return out;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let java_exe = PathBuf::from(line);
        if java_exe.is_file() {
            if let Some(home) = java_home_from_java_exe(&java_exe) {
                out.push(JvmCandidate {
                    home,
                    java_exe: Some(java_exe),
                    source: DiscoverySource::WhereCommand,
                    detail: "where java".to_string(),
                });
            }
        }
    }

    out
}
