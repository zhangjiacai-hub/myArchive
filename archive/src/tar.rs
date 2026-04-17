use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::Path;
use tar::{Archive, Builder};
use walkdir::WalkDir;

/// 将文件或目录打包为 tar 文件
pub fn compress(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源路径不存在: {}", source);
    }

    let file = File::create(output).with_context(|| format!("无法创建输出文件: {}", output))?;
    let mut builder = Builder::new(file);

    if source_path.is_file() {
        let name = source_path.file_name().unwrap();
        builder.append_path_with_name(source_path, name)?;
    } else {
        for entry in WalkDir::new(source_path) {
            let entry = entry?;
            let path = entry.path();
            let rel_path = path.strip_prefix(source_path).unwrap();

            if rel_path.as_os_str().is_empty() {
                continue;
            }

            if path.is_file() {
                builder.append_path_with_name(path, rel_path)?;
            } else if path.is_dir() {
                builder.append_dir(rel_path, path)?;
            }
        }
    }

    builder.finish()?;
    println!("已打包到: {}", output);
    Ok(())
}

/// 解包 tar 文件到指定目录
pub fn extract(source: &str, output: &str) -> Result<()> {
    let file = File::open(source).with_context(|| format!("无法打开 tar 文件: {}", source))?;
    let mut archive = Archive::new(file);
    let output_path = Path::new(output);

    fs::create_dir_all(output_path)?;
    archive.unpack(output_path)?;

    println!("已解包到: {}", output);
    Ok(())
}

/// 列出 tar 文件中的内容
pub fn list_contents(source: &str) -> Result<()> {
    let file = File::open(source).with_context(|| format!("无法打开 tar 文件: {}", source))?;
    let mut archive = Archive::new(file);

    println!("{:<10} {:<12} {}", "大小", "类型", "文件名");
    println!("{}", "-".repeat(60));

    for entry in archive.entries()? {
        let entry = entry?;
        let size = entry.size();
        let kind = match entry.header().entry_type() {
            tar::EntryType::Regular => "文件",
            tar::EntryType::Directory => "目录",
            tar::EntryType::Symlink => "符号链接",
            tar::EntryType::Link => "硬链接",
            _ => "其他",
        };
        let path = entry.path()?;
        println!("{:<10} {:<12} {}", size, kind, path.display());
    }

    Ok(())
}
