mod cli;
mod fetch;
mod schema;
mod validate;

pub type Result<T> = std::result::Result<T, String>;

pub use cli::run;
pub use fetch::download_report;
pub use schema::{Schema, report_end_from_filename, schema_for_report};
pub use validate::{ValidationResult, validate_csv};
