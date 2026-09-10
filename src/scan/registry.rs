use crate::model::{DiscoverySource, JvmCandidate};
use crate::util::{expand_env, java_home_from_java_exe, parse_command_line};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::{RegKey, HKEY};

pub fn scan_registry() -> Vec<JvmCandidate> {
    let mut out = Vec::new();

    scan_oracle_javasoft(&mut out);
    scan_vendor_roots(&mut out);
    scan_uninstall_java(&mut out);
    scan_jar_association(&mut out);

    out
}

fn scan_oracle_javasoft(out: &mut Vec<JvmCandidate>) {
    let roots = [
        (HKEY_LOCAL_MACHINE, "HKLM"),
        (HKEY_CURRENT_USER, "HKCU"),
    ];
    let wow = ["", r"SOFTWARE\WOW6432Node\"];
    let products = ["JavaSoft\\JRE", "JavaSoft\\JDK", "JavaSoft\\Java Development Kit"];

    for (hive, hive_label) in roots {
        for wow_prefix in wow {
            for product in products {
                let path = format!(r"SOFTWARE\{wow_prefix}{product}");
                read_versioned_java_homes(
                    out,
                    hive,
                    &path,
                    &format!("{hive_label}\\{path}"),
                );
            }
        }
    }
}

fn scan_vendor_roots(out: &mut Vec<JvmCandidate>) {
    let vendors: &[(&str, &str, &[&str])] = &[
        (
            r"SOFTWARE\Eclipse Adoptium\JDK",
            "Eclipse Adoptium",
            &["Path", "JavaHome", "InstallationPath"],
        ),
        (
            r"SOFTWARE\Eclipse Foundation\JDK",
            "Eclipse Foundation",
            &["Path", "JavaHome"],
        ),
        (
            r"SOFTWARE\Microsoft\JDK",
            "Microsoft",
            &["Path", "JavaHome"],
        ),
        (
            r"SOFTWARE\Azul Systems\Zulu",
            "Azul Zulu",
            &["InstallationPath", "JavaHome", "Path"],
        ),
        (
            r"SOFTWARE\BellSoft\Liberica",
            "BellSoft Liberica",
            &["InstallationPath", "JavaHome", "Path"],
        ),
        (
            r"SOFTWARE\AdoptOpenJDK\JDK",
            "AdoptOpenJDK",
            &["Path", "JavaHome"],
        ),
        (
            r"SOFTWARE\Semeru",
            "IBM Semeru",
            &["Path", "JavaHome"],
        ),
        (
            r"SOFTWARE\Amazon Corretto",
            "Amazon Corretto",
            &["Path", "JavaHome"],
        ),
    ];

    let hives = [
        (HKEY_LOCAL_MACHINE, "HKLM"),
        (HKEY_CURRENT_USER, "HKCU"),
    ];
    let wow = ["", r"SOFTWARE\WOW6432Node\"];

    for (hive, hive_label) in hives {
        for wow_prefix in wow {
            for (rel, vendor, value_names) in vendors {
                let path = if wow_prefix.is_empty() {
                    rel.to_string()
                } else {
                    let suffix = rel.strip_prefix("SOFTWARE\\").unwrap_or(rel);
                    format!("{wow_prefix}{suffix}")
                };
                read_vendor_versions(
                    out,
                    hive,
                    &path,
                    vendor,
                    value_names,
                    &format!("{hive_label}\\{path}"),
                );
            }
        }
    }
}

fn scan_uninstall_java(out: &mut Vec<JvmCandidate>) {
    let uninstall_paths = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for path in uninstall_paths {
        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
            for name in key.enum_keys().filter_map(Result::ok) {
                if let Ok(sub) = key.open_subkey(&name) {
                    let display: String = sub.get_value("DisplayName").unwrap_or_default();
                    if !looks_like_java_product(&display) {
                        continue;
                    }
                    if let Ok(install_loc) = sub.get_value::<String, _>("InstallLocation") {
                        let home = PathBuf::from(expand_env(install_loc.trim()));
                        out.push(JvmCandidate {
                            home: home.clone(),
                            java_exe: None,
                            source: DiscoverySource::Registry,
                            detail: format!(
                                r"HKLM\{path}\{name} InstallLocation ({display})"
                            ),
                        });
                    }
                }
            }
        }
    }
}

fn scan_jar_association(out: &mut Vec<JvmCandidate>) {
    let keys = [
        (HKEY_CLASSES_ROOT, r"jarfile\shell\open\command"),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Classes\jarfile\shell\open\command",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Classes\jarfile\shell\open\command",
        ),
        (
            HKEY_CLASSES_ROOT,
            r"Applications\java.exe\shell\open\command",
        ),
    ];

    for (hive, subkey) in keys {
        if let Ok(key) = RegKey::predef(hive).open_subkey(subkey) {
            if let Ok(command) = key.get_value::<String, _>("") {
                if let Some(java_exe) = parse_command_line(&command) {
                    if let Some(home) = java_home_from_java_exe(&java_exe) {
                        out.push(JvmCandidate {
                            home,
                            java_exe: Some(java_exe),
                            source: DiscoverySource::FileAssociation,
                            detail: crate::i18n::detail_registry(hive_label(hive), subkey),
                        });
                    }
                }
            }
        }
    }
}

fn read_versioned_java_homes(
    out: &mut Vec<JvmCandidate>,
    hive: HKEY,
    subkey: &str,
    detail_prefix: &str,
) {
    let Ok(key) = RegKey::predef(hive).open_subkey(subkey) else {
        return;
    };

    for version in key.enum_keys().filter_map(Result::ok) {
        if version.eq_ignore_ascii_case("CurrentVersion") {
            continue;
        }
        if let Ok(ver_key) = key.open_subkey(&version) {
            for value_name in ["JavaHome", "InstallationPath", "Path"] {
                if let Ok(raw) = ver_key.get_value::<String, _>(value_name) {
                    let home = PathBuf::from(expand_env(raw.trim()));
                    out.push(JvmCandidate {
                        home,
                        java_exe: None,
                        source: DiscoverySource::Registry,
                        detail: format!("{detail_prefix}\\{version}\\{value_name}"),
                    });
                }
            }
        }
    }
}

fn read_vendor_versions(
    out: &mut Vec<JvmCandidate>,
    hive: HKEY,
    subkey: &str,
    vendor: &str,
    value_names: &[&str],
    detail_prefix: &str,
) {
    let Ok(key) = RegKey::predef(hive).open_subkey(subkey) else {
        return;
    };

    for version in key.enum_keys().filter_map(Result::ok) {
        if let Ok(ver_key) = key.open_subkey(&version) {
            for value_name in value_names {
                if let Ok(raw) = ver_key.get_value::<String, _>(*value_name) {
                    let home = PathBuf::from(expand_env(raw.trim()));
                    out.push(JvmCandidate {
                        home,
                        java_exe: None,
                        source: DiscoverySource::Registry,
                        detail: format!("{detail_prefix}\\{vendor}\\{version}\\{value_name}"),
                    });
                }
            }
        }
    }
}

fn looks_like_java_product(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [
        "java",
        "jdk",
        "jre",
        "openjdk",
        "temurin",
        "corretto",
        "zulu",
        "liberica",
        "graalvm",
        "semeru",
        "adoptium",
    ]
    .iter()
    .any(|k| lower.contains(k))
}

fn hive_label(hive: HKEY) -> &'static str {
    if hive == HKEY_LOCAL_MACHINE {
        "HKLM"
    } else if hive == HKEY_CURRENT_USER {
        "HKCU"
    } else if hive == HKEY_CLASSES_ROOT {
        "HKCR"
    } else {
        "REG"
    }
}
