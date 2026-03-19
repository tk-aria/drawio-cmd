use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn test_extract_outputs_xml_to_stdout() {
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("extract")
        .arg("tests/fixtures/simple_embedded.png")
        .assert()
        .success()
        .stdout(predicate::str::contains("<mxfile"));
}

#[test]
fn test_extract_outputs_xml_to_file() {
    let output = NamedTempFile::new().unwrap();
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("extract")
        .arg("tests/fixtures/simple_embedded.png")
        .arg("-o")
        .arg(output.path())
        .assert()
        .success();
    let content = std::fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("<mxfile"));
}

#[test]
fn test_extract_plain_png_fails() {
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("extract")
        .arg("tests/fixtures/plain.png")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No draw.io XML found"));
}

#[test]
fn test_extract_nonexistent_file_fails() {
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("extract")
        .arg("nonexistent.png")
        .assert()
        .failure();
}

#[test]
fn test_embed_and_reextract_roundtrip() {
    let output_png = NamedTempFile::new().unwrap();

    // embed: simple.drawio + plain.png → output.png
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("embed")
        .arg("tests/fixtures/simple.drawio")
        .arg("tests/fixtures/plain.png")
        .arg("-o")
        .arg(output_png.path())
        .assert()
        .success();

    // extract: output.png → stdout に XML
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("extract")
        .arg(output_png.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"))
        .stdout(predicate::str::contains("<mxfile"));
}

#[test]
fn test_help_output() {
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("extract"))
        .stdout(predicate::str::contains("embed"));
}

#[test]
fn test_version_output() {
    Command::cargo_bin("drawio-tools")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("drawio-tools"));
}
