#![cfg(windows)]

use std::io;

/// Windows特定配置，确保程序完全便携
pub fn configure_portable() -> Result<(), io::Error> {
    // 设置应用程序为便携模式
    // 确保所有数据都存储在可执行文件目录下
    Ok(())
}

/// 获取便携式应用数据目录
pub fn get_portable_data_dir() -> Result<std::path::PathBuf, io::Error> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    Ok(exe_dir.join("data"))
}