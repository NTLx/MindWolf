fn main() {
    // 确保静态链接和嵌入所有资源
    #[cfg(windows)]
    {
        // 强制使用静态链接，避免外部依赖
        println!("cargo:rustc-env=RUSTFLAGS=-C target-feature=+crt-static");
        // 嵌入Windows资源文件
        if std::path::Path::new("resources.rc").exists() {
            println!("cargo:rerun-if-changed=resources.rc");
        }
        // 嵌入清单文件
        println!("cargo:rerun-if-changed=build.rs");
    }
    
    tauri_build::build()
}
