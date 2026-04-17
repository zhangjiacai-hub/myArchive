use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// 在临时目录中创建测试用的源文件和源目录
fn setup_test_data(tmp: &TempDir) -> (String, String) {
    // 创建单文件
    let file_path = tmp.path().join("hello.txt");
    fs::write(&file_path, "Hello, Archive!\n这是一个测试文件。\n").unwrap();

    // 创建目录结构
    let dir_path = tmp.path().join("testdir");
    fs::create_dir_all(dir_path.join("sub")).unwrap();
    fs::write(dir_path.join("a.txt"), "file a content\n").unwrap();
    fs::write(dir_path.join("sub").join("b.txt"), "file b content\n").unwrap();

    (
        file_path.to_string_lossy().to_string(),
        dir_path.to_string_lossy().to_string(),
    )
}

fn archive_cmd() -> Command {
    Command::cargo_bin("archive").unwrap()
}

// ==================== ZIP ====================

#[test]
fn test_zip_file_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (file_path, _) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.zip");
    let extract_dir = tmp.path().join("zip_extract");

    // 压缩
    archive_cmd()
        .args(["compress", &file_path, "-f", "zip", "-o"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("已压缩到"));

    // 列表
    archive_cmd()
        .args(["list"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("hello.txt"));

    // 解压
    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("已解压到"));

    let content = fs::read_to_string(extract_dir.join("hello.txt")).unwrap();
    assert!(content.contains("Hello, Archive!"));
}

#[test]
fn test_zip_dir_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (_, dir_path) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("dir.zip");
    let extract_dir = tmp.path().join("zip_dir_extract");

    archive_cmd()
        .args(["compress", &dir_path, "-f", "zip", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success();

    assert!(extract_dir.join("a.txt").exists());
    assert!(extract_dir.join("sub").join("b.txt").exists());
    let content = fs::read_to_string(extract_dir.join("sub").join("b.txt")).unwrap();
    assert_eq!(content, "file b content\n");
}

// ==================== TAR ====================

#[test]
fn test_tar_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (_, dir_path) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.tar");
    let extract_dir = tmp.path().join("tar_extract");

    archive_cmd()
        .args(["compress", &dir_path, "-f", "tar", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["list"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success();

    let content = fs::read_to_string(extract_dir.join("a.txt")).unwrap();
    assert_eq!(content, "file a content\n");
}

// ==================== GZ (单文件) ====================

#[test]
fn test_gz_file_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (file_path, _) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("hello.txt.gz");
    let output_path = tmp.path().join("hello_restored.txt");

    archive_cmd()
        .args(["compress", &file_path, "-f", "gz", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&output_path)
        .assert()
        .success();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello, Archive!"));
}

// ==================== TAR.GZ ====================

#[test]
fn test_tar_gz_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (_, dir_path) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.tar.gz");
    let extract_dir = tmp.path().join("targz_extract");

    archive_cmd()
        .args(["compress", &dir_path, "-f", "tar-gz", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["list"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success();

    let content = fs::read_to_string(extract_dir.join("sub").join("b.txt")).unwrap();
    assert_eq!(content, "file b content\n");
}

// ==================== BZ2 (单文件) ====================

#[test]
fn test_bz2_file_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (file_path, _) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("hello.txt.bz2");
    let output_path = tmp.path().join("hello_bz2_restored.txt");

    archive_cmd()
        .args(["compress", &file_path, "-f", "bz2", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&output_path)
        .assert()
        .success();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello, Archive!"));
}

// ==================== TAR.BZ2 ====================

#[test]
fn test_tar_bz2_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (_, dir_path) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.tar.bz2");
    let extract_dir = tmp.path().join("tarbz2_extract");

    archive_cmd()
        .args(["compress", &dir_path, "-f", "tar-bz2", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["list"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success();

    let content = fs::read_to_string(extract_dir.join("a.txt")).unwrap();
    assert_eq!(content, "file a content\n");
}

// ==================== XZ (单文件) ====================

#[test]
fn test_xz_file_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (file_path, _) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("hello.txt.xz");
    let output_path = tmp.path().join("hello_xz_restored.txt");

    archive_cmd()
        .args(["compress", &file_path, "-f", "xz", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&output_path)
        .assert()
        .success();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello, Archive!"));
}

// ==================== TAR.XZ ====================

#[test]
fn test_tar_xz_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (_, dir_path) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.tar.xz");
    let extract_dir = tmp.path().join("tarxz_extract");

    archive_cmd()
        .args(["compress", &dir_path, "-f", "tar-xz", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["list"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success();

    let content = fs::read_to_string(extract_dir.join("sub").join("b.txt")).unwrap();
    assert_eq!(content, "file b content\n");
}

// ==================== 7Z ====================

#[test]
fn test_7z_compress_extract() {
    let tmp = TempDir::new().unwrap();
    let (_, dir_path) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.7z");
    let extract_dir = tmp.path().join("sevenz_extract");

    archive_cmd()
        .args(["compress", &dir_path, "-f", "7z", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    archive_cmd()
        .args(["list"])
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"));

    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&extract_dir)
        .assert()
        .success();

    let content = fs::read_to_string(extract_dir.join("a.txt")).unwrap();
    assert_eq!(content, "file a content\n");
}

// ==================== RAR (仅解压，需要有 rar 文件才能测试) ====================

#[test]
fn test_rar_compress_should_fail() {
    // RAR 不支持压缩，应该报错
    let tmp = TempDir::new().unwrap();
    let (file_path, _) = setup_test_data(&tmp);
    let archive_path = tmp.path().join("out.rar");

    archive_cmd()
        .args(["compress", &file_path, "-f", "rar", "-o"])
        .arg(&archive_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("RAR"));
}

// ==================== 格式自动检测 ====================

#[test]
fn test_auto_detect_format() {
    let tmp = TempDir::new().unwrap();
    let (file_path, _) = setup_test_data(&tmp);

    // 先用 gz 压缩
    let archive_path = tmp.path().join("auto.gz");
    archive_cmd()
        .args(["compress", &file_path, "-f", "gz", "-o"])
        .arg(&archive_path)
        .assert()
        .success();

    // 解压时不指定格式，自动检测 .gz
    let output_path = tmp.path().join("auto_restored.txt");
    archive_cmd()
        .args(["extract"])
        .arg(&archive_path)
        .args(["-o"])
        .arg(&output_path)
        .assert()
        .success();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("Hello, Archive!"));
}

// ==================== 错误处理 ====================

#[test]
fn test_compress_nonexistent_source() {
    archive_cmd()
        .args(["compress", "/nonexistent/path", "-f", "zip"])
        .assert()
        .failure();
}

#[test]
fn test_extract_nonexistent_source() {
    archive_cmd()
        .args(["extract", "/nonexistent/file.zip"])
        .assert()
        .failure();
}

#[test]
fn test_extract_unknown_format() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("data.unknown");
    fs::write(&file_path, "some data").unwrap();

    archive_cmd()
        .args(["extract"])
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("无法自动检测格式"));
}
