//! # Tests for folder scanning

use crate::scanner::scan_folder;
use std::fs;
use std::path::Path;

fn create_test_dir(name: &str) -> std::path::PathBuf {
  let dir = std::env::temp_dir().join(format!("dsntk_test_{name}_{}", std::process::id()));
  let _ = fs::remove_dir_all(&dir);
  fs::create_dir_all(&dir).unwrap();
  dir
}

#[test]
fn _0001() {
  // Folder scan resolves type across multiple files
  let dir = create_test_dir("scan_multi");
  fs::write(dir.join("person.ts"), r#"export interface Person { name: string; }"#).unwrap();
  fs::write(dir.join("status.json"), r#"{ "$id": "Status", "type": "string", "enum": ["a","b"] }"#).unwrap();

  let registry = scan_folder(&dir).unwrap();
  assert!(registry.get("Person").is_some());
  assert!(registry.get("Status").is_some());

  let _ = fs::remove_dir_all(&dir);
}

#[test]
fn _0002() {
  // Missing type name in folder errors clearly
  let dir = create_test_dir("scan_missing");
  fs::write(dir.join("person.ts"), r#"export interface Person { name: string; }"#).unwrap();

  let registry = scan_folder(&dir).unwrap();
  let result = registry.resolve("NonExistent");
  assert!(result.is_err());
  let err_msg = result.unwrap_err().to_string();
  assert!(err_msg.contains("not found"));

  let _ = fs::remove_dir_all(&dir);
}

#[test]
fn _0003() {
  // Ambiguous type across folder errors with both paths
  let dir = create_test_dir("scan_ambig");
  fs::write(dir.join("a.ts"), r#"export interface Dup { x: number; }"#).unwrap();
  fs::write(dir.join("b.ts"), r#"export interface Dup { y: string; }"#).unwrap();

  let result = scan_folder(&dir);
  assert!(result.is_err());
  let err_msg = result.unwrap_err().to_string();
  assert!(err_msg.contains("ambiguous"));

  let _ = fs::remove_dir_all(&dir);
}

#[test]
fn _0004() {
  // Non-schema files are ignored
  let dir = create_test_dir("scan_ignore");
  fs::write(dir.join("readme.md"), "# Hello").unwrap();
  fs::write(dir.join("code.rs"), "fn main() {}").unwrap();
  fs::write(dir.join("person.ts"), r#"export interface Person { name: string; }"#).unwrap();

  let registry = scan_folder(&dir).unwrap();
  assert_eq!(registry.len(), 1);

  let _ = fs::remove_dir_all(&dir);
}

#[test]
fn _0005() {
  // Non-existent directory errors
  let result = scan_folder(Path::new("/nonexistent/path"));
  assert!(result.is_err());
}

#[test]
fn _0006() {
  // Empty directory returns empty registry
  let dir = create_test_dir("scan_empty");
  let registry = scan_folder(&dir).unwrap();
  assert!(registry.is_empty());
  let _ = fs::remove_dir_all(&dir);
}
