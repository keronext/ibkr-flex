use crate::Result;
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
pub struct Schema {
    date_start: String,
    date_end: Option<String>,
    pub version_number: String,
    pub(crate) sections: Vec<String>,
    pub(crate) columns: Vec<String>,
}

pub fn schema_for_report(
    schema_dir: &Path,
    query_type: &str,
    report_date: NaiveDate,
) -> Result<Schema> {
    let type_dir = schema_dir.join(query_type);
    let entries = fs::read_dir(&type_dir)
        .map_err(|error| format!("Impossible de lire {} : {error}", type_dir.display()))?;
    let mut schema_paths = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    schema_paths.sort();

    let mut matches = Vec::new();
    for path in schema_paths {
        let schema: Schema = read_json(&path)?;
        if schema_covers_report(&schema, report_date)? {
            matches.push((path, schema));
        }
    }

    match matches.len() {
        1 => Ok(matches.pop().expect("one schema").1),
        0 => Err(format!(
            "Aucun schéma applicable au rapport {} dans {}.",
            report_date.format("%Y-%m-%d"),
            type_dir.display()
        )),
        _ => Err(format!(
            "Plusieurs schémas sont applicables au rapport {} dans {} : les périodes de validité se chevauchent.",
            report_date.format("%Y-%m-%d"),
            type_dir.display()
        )),
    }
}

pub fn report_end_from_filename(file: &Path) -> Result<NaiveDate> {
    if file.extension().and_then(|extension| extension.to_str()) != Some("csv") {
        return Err(
            "Le fichier CSV doit être nommé YYYY, YYYY-MM, YYYY-MM-DD ou <début>_to_<fin>.csv."
                .to_owned(),
        );
    }
    let period = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("Le fichier CSV doit contenir une période valide.")?;
    if let Some((date_start, date_end)) = period.split_once("_to_") {
        let date_start = parse_day(date_start)?;
        let date_end = parse_day(date_end)?;
        if date_end < date_start {
            return Err(
                "La date de fin du fichier CSV doit être postérieure à la date de début."
                    .to_owned(),
            );
        }
        return Ok(date_end);
    }
    match period.len() {
        4 => {
            let year = period
                .parse::<i32>()
                .map_err(|_| "Le fichier CSV doit contenir une période valide.".to_owned())?;
            NaiveDate::from_ymd_opt(year, 12, 31)
                .ok_or("Le fichier CSV doit contenir une période valide.".to_owned())
        }
        7 => {
            let month = NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d")
                .map_err(|_| "Le fichier CSV doit contenir une période valide.".to_owned())?;
            Ok(last_day(month))
        }
        10 => parse_day(period),
        _ => Err("Le fichier CSV doit contenir une période valide.".to_owned()),
    }
}

pub(crate) fn read_json<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<T> {
    serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())
}

fn schema_covers_report(schema: &Schema, report_date: NaiveDate) -> Result<bool> {
    if schema.version_number.trim().is_empty() {
        return Err("version_number ne doit pas être vide.".to_owned());
    }
    let date_start = NaiveDate::parse_from_str(&schema.date_start, "%Y-%m-%d")
        .map_err(|_| "date_start doit être au format YYYY-MM-DD.".to_owned())?;
    let date_end = if let Some(date_end) = &schema.date_end {
        let date_end = NaiveDate::parse_from_str(date_end, "%Y-%m-%d")
            .map_err(|_| "date_end doit être au format YYYY-MM-DD.".to_owned())?;
        if date_end < date_start {
            return Err("date_end doit être postérieure ou égale à date_start.".to_owned());
        }
        Some(date_end)
    } else {
        None
    };
    Ok(report_date >= date_start && date_end.is_none_or(|date| report_date <= date))
}

fn parse_day(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| "Le fichier CSV doit contenir une période valide.".to_owned())
}

fn last_day(month: NaiveDate) -> NaiveDate {
    let next = if month.month() == 12 {
        NaiveDate::from_ymd_opt(month.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(month.year(), month.month() + 1, 1)
    };
    next.expect("valid month").pred_opt().expect("valid day")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_the_end_date_from_supported_filenames() {
        assert_eq!(
            report_end_from_filename(Path::new("2026.csv")).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        );
        assert_eq!(
            report_end_from_filename(Path::new("2026-08.csv")).unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 31).unwrap()
        );
        assert_eq!(
            report_end_from_filename(Path::new("2025-08-03.csv")).unwrap(),
            NaiveDate::from_ymd_opt(2025, 8, 3).unwrap()
        );
        assert_eq!(
            report_end_from_filename(Path::new("2025-08-03_to_2025-08-10.csv")).unwrap(),
            NaiveDate::from_ymd_opt(2025, 8, 10).unwrap()
        );
    }
}
