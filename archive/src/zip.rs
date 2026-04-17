use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io;
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// 将文件或目录压缩为 zip 文件
pub fn compress(source: &str, output: &str) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("源路径不存在: {}", source);
    }

    let file = File::create(output).with_context(|| format!("无法创建输出文件: {}", output))?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

    if source_path.is_file() {
        let name = source_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        zip.start_file(&name, options)?;
        let mut f = File::open(source_path)?;
        io::copy(&mut f, &mut zip)?;
    } else {
        for entry in WalkDir::new(source_path) {
            let entry = entry?;
            let path = entry.path();
            let rel_path = path
                .strip_prefix(source_path)
                .unwrap()
                .to_string_lossy()
                .to_string();

            if rel_path.is_empty() {
                continue;
            }

            if path.is_dir() {
                zip.add_directory(&format!("{}/", rel_path), options)?;
            } else {
                zip.start_file(&rel_path, options)?;
                let mut f = File::open(path)?;
                io::copy(&mut f, &mut zip)?;
            }
        }
    }

    zip.finish()?;
    println!("已压缩到: {}", output);
    Ok(())
}

/// 解压 zip 文件到指定目录
pub fn extract(source: &str, output: &str) -> Result<()> {
    let file = File::open(source).with_context(|| format!("无法打开 zip 文件: {}", source))?;
    let mut archive = ZipArchive::new(file)?;
    let output_path = Path::new(output);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out_path = output_path.join(entry.mangled_name());

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            io::copy(&mut entry, &mut outfile)?;
        }
    }

    println!("已解压到: {}", output);
    Ok(())
}

/// 列出 zip 文件中的内容
pub fn list_contents(source: &str) -> Result<()> {
    let file = File::open(source).with_context(|| format!("无法打开 zip 文件: {}", source))?;
    let mut archive = ZipArchive::new(file)?;

    println!("{:<10} {:<20} {}", "大小", "修改时间", "文件名");
    println!("{}", "-".repeat(60));

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let time_str = match entry.last_modified() {
            Some(dt) => format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute()
            ),
            None => "N/A".to_string(),
        };
        println!("{:<10} {:<20} {}", entry.size(), time_str, entry.name());
    }

    Ok(())
}
