use anyhow::{Context, Result};
use bzip2::Compression;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use tar::{Archive, Builder};
use walkdir::WalkDir;

/// 将文件 bz2 压缩（单文件压缩）
pub fn compress_file(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源路径不存在: {}", source);
    }
    if !source_path.is_file() {
        anyhow::bail!("bz2 只支持单文件压缩，目录请使用 tar.bz2 格式: {}", source);
    }

    let mut input =
        File::open(source_path).with_context(|| format!("无法打开源文件: {}", source))?;
    let out_file = File::create(output).with_context(|| format!("无法创建输出文件: {}", output))?;
    let mut encoder = BzEncoder::new(out_file, Compression::default());

    io::copy(&mut input, &mut encoder)?;
    encoder.finish()?;

    println!("已压缩到: {}", output);
    Ok(())
}

/// 解压 bz2 文件（单文件解压）
pub fn decompress_file(source: &str, output: &str) -> Result<()> {
    let input = File::open(source).with_context(|| format!("无法打开 bz2 文件: {}", source))?;
    let mut decoder = BzDecoder::new(input);
    let mut out_file =
        File::create(output).with_context(|| format!("无法创建输出文件: {}", output))?;

    io::copy(&mut decoder, &mut out_file)?;

    println!("已解压到: {}", output);
    Ok(())
}

/// 将文件或目录打包并压缩为 tar.bz2 文件
pub fn compress_tar(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源路径不存在: {}", source);
    }

    let file = File::create(output).with_context(|| format!("无法创建输出文件: {}", output))?;
    let encoder = BzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

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

    let encoder = builder.into_inner()?;
    encoder.finish()?;

    println!("已压缩到: {}", output);
    Ok(())
}

/// 解压 tar.bz2 文件到指定目录
pub fn extract_tar(source: &str, output: &str) -> Result<()> {
    let file = File::open(source).with_context(|| format!("无法打开 tar.bz2 文件: {}", source))?;
    let decoder = BzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let output_path = Path::new(output);

    fs::create_dir_all(output_path)?;
    archive.unpack(output_path)?;

    println!("已解压到: {}", output);
    Ok(())
}

/// 列出 tar.bz2 文件中的内容
pub fn list_tar_contents(source: &str) -> Result<()> {
    let file = File::open(source).with_context(|| format!("无法打开 tar.bz2 文件: {}", source))?;
    let decoder = BzDecoder::new(file);
    let mut archive = Archive::new(decoder);

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
