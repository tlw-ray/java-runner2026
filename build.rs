fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/java-runner2026.ico");
        res.set("ProductName", "java-runner2026");
        res.set("FileDescription", "Windows JVM scanner and Java launcher");
        res.set("CompanyName", "java-runner2026");
        res.set("LegalCopyright", "java-runner2026");
        if let Err(e) = res.compile() {
            eprintln!("winres error: {e}");
            std::process::exit(1);
        }
    }
}
