fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        
        // 设置 icon（如果存在）
        if std::path::Path::new("icon.ico").exists() {
            if let Err(e) = res.set_icon("icon.ico") {
                eprintln!("Failed to set icon: {}", e);
            } else {
                println!("Icon set: icon.ico");
            }
        } else {
            println!("No icon.ico found, skipping...");
        }
        
        // 设置版本信息
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, winres::FileVersionInfo::FILEVERSION);
        
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resource: {}", e);
        }
    }
}
