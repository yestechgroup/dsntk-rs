//! # Error definitions

use dsntk_common::{DsntkError, ToErrorMessage};

/// Command-line action error.
#[derive(ToErrorMessage)]
struct ActionError(String);

/// Error related to creating directory.
pub fn err_create_directory(path: &str, reason: &str) -> DsntkError {
  ActionError(format!("creating directory '{path}' failed with reason: {reason}")).into()
}

/// Error related to saving a file.
pub fn err_save_file(path: &str, reason: &str) -> DsntkError {
  ActionError(format!("saving file '{path}' failed with reason: {reason}")).into()
}

/// Error when a non-empty directory already exists.
pub fn err_directory_not_empty(path: &str) -> DsntkError {
  ActionError(format!("directory '{path}' is not empty, use --force to overwrite")).into()
}

/// Error when a template is not found.
pub fn err_template_not_found(name: &str) -> DsntkError {
  ActionError(format!("unknown template '{name}', use 'dsntk new --list' to see available templates")).into()
}

/// Error when interactive mode is used without a TTY.
pub fn err_not_a_tty() -> DsntkError {
  ActionError("interactive template picker requires a TTY; pass a template name explicitly (e.g. 'dsntk new loan-eligibility')".to_string()).into()
}
