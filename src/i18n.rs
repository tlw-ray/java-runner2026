use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    ZhCn,
    En,
}

static LANGUAGE: OnceLock<Language> = OnceLock::new();

pub fn init() {
    let _ = LANGUAGE.set(detect_language());
}

pub fn language() -> Language {
    *LANGUAGE.get().unwrap_or(&Language::En)
}

/// Override locale for tests: `JAVA_RUNNER_LANG=zh-CN` or `en`.
pub fn detect_language() -> Language {
    if let Ok(tag) = std::env::var("JAVA_RUNNER_LANG") {
        return language_from_tag(&tag);
    }
    language_from_tag(&detect_system_locale_tag())
}

pub fn language_from_tag(tag: &str) -> Language {
    let normalized = tag.trim().to_ascii_lowercase().replace('-', "_");
    if normalized.starts_with("zh") {
        Language::ZhCn
    } else {
        Language::En
    }
}

fn detect_system_locale_tag() -> String {
    #[cfg(windows)]
    {
        if let Some(tag) = windows_user_locale_name() {
            return tag;
        }
    }
    std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_else(|_| "en-US".to_string())
}

#[cfg(windows)]
fn windows_user_locale_name() -> Option<String> {
    unsafe {
        extern "system" {
            fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
        }
        let mut buf = [0u16; 85];
        let len = GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32);
        if len > 1 {
            Some(String::from_utf16_lossy(&buf[..len as usize - 1]))
        } else {
            None
        }
    }
}

pub struct Messages {
    pub about: &'static str,
    pub help_json: &'static str,
    pub help_scan_only: &'static str,
    pub help_verbose: &'static str,
    pub help_config: &'static str,
    pub help_select: &'static str,
    pub help_no_pause: &'static str,
    pub error_prefix: &'static str,
    pub scanning: &'static str,
    pub no_jvm: &'static str,
    pub label_version: &'static str,
    pub label_product: &'static str,
    pub label_runtime: &'static str,
    pub label_vm: &'static str,
    pub version_probe_failed: &'static str,
    pub discovery_sources: &'static str,
    pub prompt_select_jvm: &'static str,
    pub prompt_invalid_number: &'static str,
    pub read_input_failed: &'static str,
    pub using_jvm: &'static str,
    pub running_command: &'static str,
    pub java_terminated_abnormally: &'static str,
    pub press_any_key: &'static str,
}

pub fn messages() -> Messages {
    match language() {
        Language::ZhCn => Messages {
            about: "扫描 Windows JVM 安装位置，选择其中一个并在当前终端运行可配置的 Java 启动命令",
            help_json: "以 JSON 格式输出扫描结果（不进入选择/启动流程）",
            help_scan_only: "仅扫描并列出 JVM，不提示选择、不启动程序",
            help_verbose: "显示每个 JVM 的发现来源详情",
            help_config: "配置文件路径（默认依次查找 java-runner.toml、config/java-runner.toml）",
            help_select: "直接选择第 N 个 JVM 并启动（序号从 1 开始，跳过交互）",
            help_no_pause: "执行完成后不等待按键（供终端脚本使用）",
            error_prefix: "错误",
            scanning: "正在扫描 Windows JVM 安装位置...",
            no_jvm: "未能检测到该机器安装的java环境，请安装后再执行本程序",
            label_version: "版本",
            label_product: "产品",
            label_runtime: "运行时",
            label_vm: "虚拟机",
            version_probe_failed: "（java -version 执行失败）",
            discovery_sources: "发现来源",
            prompt_select_jvm: "请选择要使用的 JVM（输入序号）:",
            prompt_invalid_number: "请输入有效数字。",
            read_input_failed: "读取用户输入失败",
            using_jvm: "使用 JVM",
            running_command: "执行命令",
            java_terminated_abnormally: "Java 进程异常终止",
            press_any_key: "按任意键退出...",
        },
        Language::En => Messages {
            about: "Scan Windows JVM installations, pick one, and run a configurable Java command in the current terminal",
            help_json: "Output scan results as JSON (skip selection and launch)",
            help_scan_only: "Scan and list JVMs only; do not prompt or launch",
            help_verbose: "Show discovery source details for each JVM",
            help_config: "Config file path (default: java-runner.toml, then config/java-runner.toml)",
            help_select: "Use the Nth JVM directly (1-based; skip interactive selection)",
            help_no_pause: "Do not wait for a key press after exit (for scripts)",
            error_prefix: "Error",
            scanning: "Scanning Windows JVM installations...",
            no_jvm: "No Java installation was detected on this machine. Install Java and run this program again.",
            label_version: "Version",
            label_product: "Product",
            label_runtime: "Runtime",
            label_vm: "VM",
            version_probe_failed: "(java -version failed)",
            discovery_sources: "Discovery sources",
            prompt_select_jvm: "Select a JVM (enter index):",
            prompt_invalid_number: "Enter a valid number.",
            read_input_failed: "Failed to read user input",
            using_jvm: "Using JVM",
            running_command: "Command",
            java_terminated_abnormally: "Java process terminated abnormally",
            press_any_key: "Press any key to exit...",
        },
    }
}

pub fn discovery_source_label(source: crate::model::DiscoverySource) -> &'static str {
    use crate::model::DiscoverySource;
    match (language(), source) {
        (Language::ZhCn, DiscoverySource::Registry) => "注册表",
        (Language::ZhCn, DiscoverySource::EnvironmentVariable) => "环境变量",
        (Language::ZhCn, DiscoverySource::ProgramFiles) => "Program Files",
        (Language::ZhCn, DiscoverySource::FileAssociation) => "文件关联",
        (Language::ZhCn, DiscoverySource::PathVariable) => "PATH",
        (Language::ZhCn, DiscoverySource::WhereCommand) => "where java",
        (Language::ZhCn, DiscoverySource::CommonPath) => "常见路径",
        (Language::En, DiscoverySource::Registry) => "Registry",
        (Language::En, DiscoverySource::EnvironmentVariable) => "Environment variable",
        (Language::En, DiscoverySource::ProgramFiles) => "Program Files",
        (Language::En, DiscoverySource::FileAssociation) => "File association",
        (Language::En, DiscoverySource::PathVariable) => "PATH",
        (Language::En, DiscoverySource::WhereCommand) => "where java",
        (Language::En, DiscoverySource::CommonPath) => "Common path",
    }
}

pub fn detail_env_var(name: &str) -> String {
    match language() {
        Language::ZhCn => format!("环境变量 {name}"),
        Language::En => format!("Environment variable {name}"),
    }
}

pub fn detail_path_dir(path: &str) -> String {
    match language() {
        Language::ZhCn => format!("PATH 目录: {path}"),
        Language::En => format!("PATH entry: {path}"),
    }
}

pub fn detail_common_path(path: &str) -> String {
    match language() {
        Language::ZhCn => format!("常见路径: {path}"),
        Language::En => format!("Common path: {path}"),
    }
}

pub fn detail_program_files_direct(path: &str) -> String {
    match language() {
        Language::ZhCn => format!("Program Files 直扫: {path}"),
        Language::En => format!("Program Files direct scan: {path}"),
    }
}

pub fn detail_registry(hive: &str, subkey: &str) -> String {
    match language() {
        Language::ZhCn => format!("注册表 {hive}\\{subkey}"),
        Language::En => format!("Registry {hive}\\{subkey}"),
    }
}

pub fn format_version_key_not_found(key: &str) -> String {
    match language() {
        Language::ZhCn => format!(
            "未找到版本信息包含 \"{key}\" 的 JVM，请检查 version_key 配置或安装对应 Java 版本"
        ),
        Language::En => format!(
            "No JVM whose version output contains \"{key}\". Check version_key or install the matching Java version."
        ),
    }
}

pub fn format_version_key_filtered(key: &str, count: usize) -> String {
    match language() {
        Language::ZhCn => format!("已按 version_key \"{key}\" 过滤，显示 {count} 个 JVM"),
        Language::En => format!("Filtered by version_key \"{key}\", showing {count} JVM(s)"),
    }
}

pub fn format_jvm_summary(count: usize, raw: usize, elapsed: std::time::Duration) -> String {
    match language() {
        Language::ZhCn => format!(
            "共发现 {count} 个 JVM（候选 {raw} 条，扫描耗时 {elapsed:.2?}）"
        ),
        Language::En => format!(
            "Found {count} JVM(s) ({raw} raw candidate(s), scan took {elapsed:.2?})"
        ),
    }
}

pub fn format_java_home_matched(index: usize, home: &str) -> String {
    match language() {
        Language::ZhCn => format!("已按配置 java_home 选用 [{index}] {home}"),
        Language::En => format!("Using java_home match [{index}] {home}"),
    }
}

pub fn format_java_home_not_found(hint: &str) -> String {
    match language() {
        Language::ZhCn => format!(
            "配置 java_home = \"{hint}\" 未匹配到已安装的 JVM，请安装或修改配置"
        ),
        Language::En => format!(
            "java_home = \"{hint}\" did not match any installed JVM. Install Java or update the config."
        ),
    }
}

pub fn format_single_jvm_auto(home: &str) -> String {
    match language() {
        Language::ZhCn => format!("仅发现 1 个 JVM，已自动选用 [1] {home}"),
        Language::En => format!("Only one JVM found; auto-selected [1] {home}"),
    }
}

pub fn format_invalid_selection(index: usize, max: usize) -> String {
    match language() {
        Language::ZhCn => format!("无效序号 {index}，请输入 1 到 {max} 之间的数字"),
        Language::En => format!("Invalid index {index}. Enter a number from 1 to {max}."),
    }
}

pub fn format_prompt_empty_input(max: usize) -> String {
    match language() {
        Language::ZhCn => format!("请输入 1 到 {max} 之间的序号。"),
        Language::En => format!("Enter an index from 1 to {max}."),
    }
}

pub fn format_read_config_failed(path: &str) -> String {
    match language() {
        Language::ZhCn => format!("读取配置文件 {path} 失败"),
        Language::En => format!("Failed to read config file {path}"),
    }
}

pub fn format_launch_failed(path: &str) -> String {
    match language() {
        Language::ZhCn => format!("启动 {path} 失败"),
        Language::En => format!("Failed to launch {path}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zh_cn_tag_uses_chinese() {
        assert_eq!(language_from_tag("zh-CN"), Language::ZhCn);
        assert_eq!(language_from_tag("zh_CN"), Language::ZhCn);
    }

    #[test]
    fn en_tag_uses_english() {
        assert_eq!(language_from_tag("en-US"), Language::En);
        assert_eq!(language_from_tag("de-DE"), Language::En);
    }
}
