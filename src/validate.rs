use crate::{
    Result,
    schema::{report_end_from_filename, schema_for_report},
};
use chrono::Utc;
use std::{fs, path::Path};

pub struct ValidationResult {
    pub schema_version: String,
    pub data_rows: usize,
    pub strict_columns: bool,
}

pub fn validate_csv(file: &Path, schema_dir: &Path, query_type: &str) -> Result<ValidationResult> {
    // Le nom du fichier indique la période couverte par le rapport, mais la
    // version du schéma dépend de la date à laquelle il est traité.
    let _ = report_end_from_filename(file)?;
    let processing_date = Utc::now().date_naive();
    let schema = schema_for_report(schema_dir, query_type, processing_date)?;
    let content = fs::read(file).map_err(|error| error.to_string())?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(content.as_slice());
    let mut header: Option<Vec<String>> = None;
    let mut data_rows = 0usize;
    let mut section_found = false;
    let flat_report = schema.sections.is_empty();

    for (record_index, record) in reader.records().enumerate() {
        let record = record.map_err(|error| format!("CSV invalide : {error}"))?;
        if flat_report {
            if record_index == 0 {
                header = Some(
                    record
                        .iter()
                        .enumerate()
                        .map(|(index, value)| {
                            if index == 0 {
                                value.trim_start_matches('\u{feff}').to_owned()
                            } else {
                                value.to_owned()
                            }
                        })
                        .collect(),
                );
            } else {
                data_rows += 1;
            }
            continue;
        }

        let section = record
            .get(0)
            .unwrap_or("")
            .trim_start_matches('\u{feff}')
            .to_ascii_lowercase();
        if !schema
            .sections
            .iter()
            .any(|expected| section.contains(&expected.to_ascii_lowercase()))
        {
            continue;
        }
        section_found = true;
        match record.get(1) {
            Some("Header") if header.is_none() => {
                header = Some(record.iter().skip(2).map(ToOwned::to_owned).collect())
            }
            Some("Data") => data_rows += 1,
            _ => {}
        }
    }

    if !flat_report && !section_found {
        return Err(format!(
            "Le CSV ne correspond pas au schéma {query_type} : section attendue absente."
        ));
    }
    let strict_columns = !schema.columns.is_empty();
    if strict_columns && header.as_deref() != Some(schema.columns.as_slice()) {
        return Err("Les colonnes CSV ne correspondent pas au schéma.".to_owned());
    }

    Ok(ValidationResult {
        schema_version: schema.version_number,
        data_rows,
        strict_columns,
    })
}
