mod bz2;
mod gz;
mod rar;
mod seven_z;
mod tar;
mod xz;
mod zip;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "archive",
    about = "文件压缩与解压工具，支持 zip/tar/gz/bz2/xz/tar.gz/tar.bz2/tar.xz/7z/rar 格式"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum Format {
    Zip,
    Tar,
    Gz,
    TarGz,
    Bz2,
    TarBz2,
    Xz,
    TarXz,
    #[value(name = "7z")]
    SevenZ,
    Rar,
}

#[derive(Subcommand)]
enum Commands {
    /// 将文件或目录压缩/打包
    Compress {
        /// 要压缩的源文件或目录路径
        source: String,
        /// 压缩格式: zip, tar, gz, bz2, xz, tar-gz, tar-bz2, tar-xz, 7z
        #[arg(short, long, value_enum, default_value = "zip")]
        format: Format,
        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 解压/解包文件
    Extract {
        /// 要解压的文件路径
        source: String,
        /// 压缩格式: zip, tar, gz, bz2, xz, tar-gz, tar-bz2, tar-xz, 7z, rar（可自动检测）
        #[arg(short, long, value_enum)]
        format: Option<Format>,
        /// 解压输出目录
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 列出压缩包内容
    List {
        /// 压缩包文件路径
        source: String,
        /// 压缩格式: zip, tar, tar-gz, tar-bz2, tar-xz, 7z, rar（可自动检测）
        #[arg(short, long, value_enum)]
        format: Option<Format>,
    },
}

/// 根据文件扩展名自动检测格式
fn detect_format(path: &str) -> Option<Format> {
    let lower = path.to_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Some(Format::TarGz)
    } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
        Some(Format::TarBz2)
    } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
        Some(Format::TarXz)
    } else if lower.ends_with(".gz") {
        Some(Format::Gz)
    } else if lower.ends_with(".bz2") {
        Some(Format::Bz2)
    } else if lower.ends_with(".xz") {
        Some(Format::Xz)
    } else if lower.ends_with(".tar") {
        Some(Format::Tar)
    } else if lower.ends_with(".zip") {
        Some(Format::Zip)
    } else if lower.ends_with(".7z") {
        Some(Format::SevenZ)
    } else if lower.ends_with(".rar") {
        Some(Format::Rar)
    } else {
        None
    }
}

/// 根据格式生成默认输出文件名
fn default_compress_output(source: &str, format: &Format) -> Result<String> {
    let p = Path::new(source);
    let stem = p.file_stem().unwrap_or_default().to_string_lossy();
    match format {
        Format::Zip => Ok(format!("{}.zip", stem)),
        Format::Tar => Ok(format!("{}.tar", stem)),
        Format::Gz => Ok(format!("{}.gz", source)),
        Format::TarGz => Ok(format!("{}.tar.gz", stem)),
        Format::Bz2 => Ok(format!("{}.bz2", source)),
        Format::TarBz2 => Ok(format!("{}.tar.bz2", stem)),
        Format::Xz => Ok(format!("{}.xz", source)),
        Format::TarXz => Ok(format!("{}.tar.xz", stem)),
        Format::SevenZ => Ok(format!("{}.7z", stem)),
        Format::Rar => anyhow::bail!("RAR 格式不支持压缩，仅支持解压"),
    }
}

/// 根据格式生成默认解压输出目录/文件名
fn default_extract_output(source: &str, format: &Format) -> String {
    let p = Path::new(source);
    match format {
        Format::Gz => {
            // file.txt.gz -> file.txt
            let name = p
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.is_empty() {
                "output".to_string()
            } else {
                name
            }
        }
        Format::TarGz => {
            // file.tar.gz -> file
            let stem = p
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let stem = stem.strip_suffix(".tar").unwrap_or(&stem).to_string();
            if stem.is_empty() {
                "output".to_string()
            } else {
                stem
            }
        }
        _ => {
            let stem = p
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if stem.is_empty() {
                "output".to_string()
            } else {
                stem
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress {
            source,
            format,
            output,
        } => {
            let output = match output {
                Some(o) => o,
                None => default_compress_output(&source, &format)?,
            };
            match format {
                Format::Zip => zip::compress(&source, &output)?,
                Format::Tar => tar::compress(&source, &output)?,
                Format::Gz => gz::compress_file(&source, &output)?,
                Format::TarGz => gz::compress_tar(&source, &output)?,
                Format::Bz2 => bz2::compress_file(&source, &output)?,
                Format::TarBz2 => bz2::compress_tar(&source, &output)?,
                Format::Xz => xz::compress_file(&source, &output)?,
                Format::TarXz => xz::compress_tar(&source, &output)?,
                Format::SevenZ => seven_z::compress(&source, &output)?,
                Format::Rar => anyhow::bail!("RAR 格式不支持压缩，仅支持解压"),
            }
        }
        Commands::Extract {
            source,
            format,
            output,
        } => {
            let format = format
                .or_else(|| detect_format(&source))
                .ok_or_else(|| anyhow::anyhow!("无法自动检测格式，请使用 -f 指定格式"))?;
            let output = output.unwrap_or_else(|| default_extract_output(&source, &format));
            match format {
                Format::Zip => zip::extract(&source, &output)?,
                Format::Tar => tar::extract(&source, &output)?,
                Format::Gz => gz::decompress_file(&source, &output)?,
                Format::TarGz => gz::extract_tar(&source, &output)?,
                Format::Bz2 => bz2::decompress_file(&source, &output)?,
                Format::TarBz2 => bz2::extract_tar(&source, &output)?,
                Format::Xz => xz::decompress_file(&source, &output)?,
                Format::TarXz => xz::extract_tar(&source, &output)?,
                Format::SevenZ => seven_z::extract(&source, &output)?,
                Format::Rar => rar::extract(&source, &output)?,
            }
        }
        Commands::List { source, format } => {
            let format = format
                .or_else(|| detect_format(&source))
                .ok_or_else(|| anyhow::anyhow!("无法自动检测格式，请使用 -f 指定格式"))?;
            match format {
                Format::Zip => zip::list_contents(&source)?,
                Format::Tar => tar::list_contents(&source)?,
                Format::TarGz => gz::list_tar_contents(&source)?,
                Format::TarBz2 => bz2::list_tar_contents(&source)?,
                Format::TarXz => xz::list_tar_contents(&source)?,
                Format::SevenZ => seven_z::list_contents(&source)?,
                Format::Rar => rar::list_contents(&source)?,
                Format::Gz | Format::Bz2 | Format::Xz => {
                    anyhow::bail!("单文件压缩格式不支持列出内容，请直接解压")
                }
            }
        }
    }

    Ok(())
}
