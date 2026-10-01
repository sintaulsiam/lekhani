fn main() {
    slint_build::compile("ui/topbar.slint").unwrap();

    // Embed Windows application icon and high-DPI manifest when building for Windows
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../data/icons/lekhani.ico");
        res.set_manifest_file("../../packaging/windows/lekhani.manifest");
        res.set("FileDescription", "Lekhani Bengali Input Method");
        res.set("ProductName", "Lekhani");
        res.set("OriginalFilename", "lekhani-gui.exe");
        res.set("LegalCopyright", "Copyright (C) 2026 Syntenieum");

        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
