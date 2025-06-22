
fn main() {
    // 编译Slint文件
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
    
    // 在Windows平台上添加资源文件
    #[cfg(windows)]
    {
        use std::path::Path;
        
        let mut res = winres::WindowsResource::new();
        
        // 检查并设置图标文件
        let icon_path = "assets/icons/icon.ico";
        if Path::new(icon_path).exists() {
            res.set_icon(icon_path);
            println!("cargo:rerun-if-changed={}", icon_path);
        } else {
            eprintln!("Warning: Icon file not found: {}", icon_path);
        }
        
        // 设置应用程序信息
        res.set("ProductName", "Extended Search");
        res.set("FileDescription", "Extended Search Application");
        res.set("CompanyName", "Your Company");
        res.set("FileVersion", "0.1.0");
        res.set("ProductVersion", "0.1.0");
        res.set("LegalCopyright", "Copyright (C) 2025");
        res.set("OriginalFilename", "ExtendedSearch.exe");
        res.set("InternalName", "ExtendedSearch");
        
        // 编译资源文件
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        } else {
            println!("cargo:rerun-if-changed=app.rc");
        }
    }
}