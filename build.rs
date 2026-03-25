fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        // Set icon (if exists)
        if std::path::Path::new("icon.ico").exists() {
            res.set_icon("icon.ico");
            println!("Icon set: icon.ico");
        } else {
            println!("No icon.ico found, skipping...");
        }

        // Set version information
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 1);

        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resource: {}", e);
        }
    }
}
