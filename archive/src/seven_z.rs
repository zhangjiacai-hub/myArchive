use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// 将文件或目录压缩为 7z 文件
pub fn compress(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源路径不存在: {}", source);
    }

    sevenz_rust2::compress_to_path(source_path, Path::new(output))
        .with_context(|| format!("压缩到 7z 失败: {} -> {}", source, output))?;

    println!("已压缩到: {}", output);
    Ok(())
}

/// 解压 7z 文件到指定目录
pub fn extract(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源文件不存在: {}", source);
    }

    let output_path = Path::new(output);
    fs::create_dir_all(output_path)?;

    sevenz_rust2::decompress_file(source_path, output_path)
        .with_context(|| format!("解压 7z 失败: {} -> {}", source, output))?;

    println!("已解压到: {}", output);
    Ok(())
}

/// 列出 7z 文件中的内容
pub fn list_contents(source: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源文件不存在: {}", source);
    }

    let archive = sevenz_rust2::Archive::open(source_path)
        .with_context(|| format!("无法读取 7z 归档: {}", source))?;

    println!("{:<10} {:<8} {}", "大小", "属性", "文件名");
    println!("{}", "-".repeat(60));

    for entry in &archive.files {
        let name = entry.name();
        let size = entry.size();
        let attr = if entry.is_directory() {
            "目录"
        } else {
            "文件"
        };
        println!("{:<10} {:<8} {}", size, attr, name);
    }

    Ok(())
}
