use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use unrar::Archive;

/// RAR 格式为专有格式，仅支持解压和列表，不支持创建压缩包

/// 解压 RAR 文件到指定目录
pub fn extract(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源文件不存在: {}", source);
    }

    let output_path = Path::new(output);
    fs::create_dir_all(output_path)?;

    let archive = Archive::new(source)
        .open_for_processing()
        .with_context(|| format!("无法打开 RAR 文件: {}", source))?;

    let mut current = Some(archive);
    while let Some(arc) = current {
        match arc.read_header() {
            Ok(Some(header)) => {
                let is_file = header.entry().is_file();
                if is_file {
                    let next = header
                        .extract_with_base(output_path)
                        .with_context(|| "解压文件失败")?;
                    current = Some(next);
                } else {
                    let next = header.skip().with_context(|| "跳过条目失败")?;
                    current = Some(next);
                }
            }
            Ok(None) => {
                current = None;
            }
            Err(e) => {
                anyhow::bail!("读取 RAR 头部失败: {}", e);
            }
        }
    }

    println!("已解压到: {}", output);
    Ok(())
}

/// 列出 RAR 文件中的内容
pub fn list_contents(source: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源文件不存在: {}", source);
    }

    let archive = Archive::new(source)
        .open_for_listing()
        .with_context(|| format!("无法打开 RAR 文件: {}", source))?;

    println!("{:<10} {:<8} {}", "大小", "属性", "文件名");
    println!("{}", "-".repeat(60));

    for entry in archive {
        let entry = entry.with_context(|| "读取 RAR 条目失败")?;
        let attr = if entry.is_directory() {
            "目录"
        } else {
            "文件"
        };
        println!(
            "{:<10} {:<8} {}",
            entry.unpacked_size,
            attr,
            entry.filename.display()
        );
    }

    Ok(())
}
