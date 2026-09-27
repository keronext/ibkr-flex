use crate::{Result, fetch::download_report, validate::validate_csv};
use chrono::{Datelike, NaiveDate, Utc};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

const IBKR_MAX_DAYS: i64 = 365;
const FLEX_QUERIES_FILE: &str = "flex-queries.toml";
const FETCH_OPTIONS: &[&str] = &[
    "--query-id",
    "--root",
    "--date",
    "--from",
    "--to",
    "--max-days",
];
const VALIDATE_OPTIONS: &[&str] = &["--type", "--schema-dir"];

struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
    filename: String,
}

pub async fn run(args: Vec<String>) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("fetch") => fetch(&args[1..]).await,
        Some("validate") => validate(&args[1..]),
        Some("list") => list(&args[1..]),
        _ => Err("Usage : ibkr-flex-cli fetch <type> [--query-id ID] [--root DIR] [--date YYYY|YYYY-MM|YYYY-MM-DD | --from YYYY-MM-DD --to YYYY-MM-DD] [--max-days 1..365] | validate --type <type> <fichier.csv> [--schema-dir DIR] | list flex".to_owned()),
    }
}

fn list(args: &[String]) -> Result<()> {
    match args {
        [resource] if resource == "flex" => list_flex(),
        _ => Err("Usage : ibkr-flex-cli list flex".to_owned()),
    }
}

fn list_flex() -> Result<()> {
    let queries = read_queries()?;
    let mut queries: Vec<_> = queries.into_iter().collect();
    queries.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    println!("Flex Queries disponibles :");
    for (query_type, query_id) in queries {
        println!("- {query_type}: {query_id}");
    }
    Ok(())
}

async fn fetch(args: &[String]) -> Result<()> {
    let query_type = args.first().ok_or("Le type de Flex Query est requis.")?;
    if query_type.starts_with("--") {
        return Err("Le type de Flex Query est requis.".to_owned());
    }
    let positional = positional_args(&args[1..], FETCH_OPTIONS)?;
    if !positional.is_empty() {
        return Err(format!(
            "Argument inattendu pour fetch : {}.",
            positional[0]
        ));
    }
    let root = option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("downloads"));
    let date_range = date_range(args)?;
    let queries = read_queries()?;
    let query_id = option(args, "--query-id")
        .map(str::to_owned)
        .or_else(|| queries.get(query_type).map(ToString::to_string))
        .ok_or_else(|| format!("Type inconnu dans {FLEX_QUERIES_FILE} : {query_type}"))?;
    let token = env::var("FLEX_WEB_SERVICE_TOKEN")
        .map_err(|_| "FLEX_WEB_SERVICE_TOKEN est requis dans .env.")?;
    if token.trim().is_empty() {
        return Err("FLEX_WEB_SERVICE_TOKEN est vide.".to_owned());
    }

    let csv = download_report(&token, &query_id, date_range.start, date_range.end).await?;
    let destination = root.join(query_type).join(&date_range.filename);
    fs::create_dir_all(destination.parent().expect("file has parent"))
        .map_err(|error| error.to_string())?;
    fs::write(&destination, csv).map_err(|error| error.to_string())?;
    println!("Rapport enregistré : {}", destination.display());
    Ok(())
}

fn validate(args: &[String]) -> Result<()> {
    let query_type = option(args, "--type").ok_or("--type est requis.")?;
    let positional = positional_args(args, VALIDATE_OPTIONS)?;
    let file = match positional.as_slice() {
        [file] => *file,
        [] => return Err("Le fichier CSV est requis.".to_owned()),
        _ => return Err("Un seul fichier CSV doit être fourni.".to_owned()),
    };
    let schema_dir = option(args, "--schema-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("schemas/flex"));
    let result = validate_csv(Path::new(file), &schema_dir, query_type)?;

    if !result.strict_columns {
        println!("WARNING: schéma sans colonnes strictes ; seule la section est validée.");
    }
    if result.data_rows == 0 {
        eprintln!("WARNING: le rapport ne contient aucune ligne de données.");
    }
    println!(
        "Validation réussie : {file} (schéma v{})",
        result.schema_version
    );
    Ok(())
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn read_queries() -> Result<HashMap<String, u64>> {
    let content = fs::read_to_string(FLEX_QUERIES_FILE).map_err(|error| error.to_string())?;
    toml::from_str(&content).map_err(|error| error.to_string())
}

fn positional_args<'a>(args: &'a [String], allowed_options: &[&str]) -> Result<Vec<&'a str>> {
    let mut positional = Vec::new();
    let mut seen_options = std::collections::HashSet::new();
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if argument.starts_with("--") {
            if !allowed_options.contains(&argument.as_str()) {
                return Err(format!("Option inconnue : {argument}."));
            }
            if !seen_options.insert(argument.as_str()) {
                return Err(format!("Option fournie plusieurs fois : {argument}."));
            }
            let value = args
                .get(index + 1)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("Une valeur est requise pour {argument}."))?;
            let _ = value;
            index += 2;
        } else {
            positional.push(argument.as_str());
            index += 1;
        }
    }
    Ok(positional)
}

fn date_range(args: &[String]) -> Result<DateRange> {
    let max_days = option(args, "--max-days")
        .map(parse_max_days)
        .transpose()?
        .unwrap_or(IBKR_MAX_DAYS);
    let date = option(args, "--date");
    let date_start = option(args, "--from");
    let date_end = option(args, "--to");
    let range = match (date, date_start, date_end) {
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            return Err("--date ne peut pas être combiné avec --from ou --to.".to_owned());
        }
        (Some(date), None, None) => parse_date_spec(date)?,
        (None, Some(date_start), Some(date_end)) => {
            let start = parse_day(date_start, "--from")?;
            let end = parse_day(date_end, "--to")?;
            let filename = if start == end {
                format!("{}.csv", start.format("%Y-%m-%d"))
            } else {
                format!(
                    "{}_to_{}.csv",
                    start.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                )
            };
            DateRange {
                start,
                end,
                filename,
            }
        }
        (None, None, None) => last_complete_month(),
        _ => return Err("--from et --to doivent être fournis ensemble.".to_owned()),
    };
    if range.end < range.start {
        return Err("La date de fin doit être postérieure ou égale à la date de début.".to_owned());
    }
    let days = (range.end - range.start).num_days() + 1;
    if days > max_days {
        return Err(format!(
            "La période demandée couvre {days} jours, au-delà de la limite de {max_days} jours."
        ));
    }
    Ok(range)
}

fn parse_date_spec(value: &str) -> Result<DateRange> {
    match value.len() {
        4 => {
            let year = value
                .parse::<i32>()
                .map_err(|_| "--date doit être YYYY, YYYY-MM ou YYYY-MM-DD.".to_owned())?;
            let start = NaiveDate::from_ymd_opt(year, 1, 1)
                .ok_or("--date doit être YYYY, YYYY-MM ou YYYY-MM-DD.")?;
            Ok(DateRange {
                start,
                end: NaiveDate::from_ymd_opt(year, 12, 31).expect("valid date"),
                filename: format!("{value}.csv"),
            })
        }
        7 => {
            let start = NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d")
                .map_err(|_| "--date doit être YYYY, YYYY-MM ou YYYY-MM-DD.".to_owned())?;
            Ok(DateRange {
                end: last_day(start),
                start,
                filename: format!("{value}.csv"),
            })
        }
        10 => {
            let date = parse_day(value, "--date")?;
            Ok(DateRange {
                start: date,
                end: date,
                filename: format!("{value}.csv"),
            })
        }
        _ => Err("--date doit être YYYY, YYYY-MM ou YYYY-MM-DD.".to_owned()),
    }
}

fn parse_max_days(value: &str) -> Result<i64> {
    let days = value
        .parse::<i64>()
        .map_err(|_| "--max-days doit être un entier entre 1 et 365.".to_owned())?;
    if !(1..=IBKR_MAX_DAYS).contains(&days) {
        return Err("--max-days doit être un entier entre 1 et 365.".to_owned());
    }
    Ok(days)
}

fn parse_day(value: &str, option_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("{option_name} doit être YYYY-MM-DD."))
}

fn last_complete_month() -> DateRange {
    let today = Utc::now().date_naive();
    let current = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("valid month");
    let start = if current.month() == 1 {
        NaiveDate::from_ymd_opt(current.year() - 1, 12, 1).expect("valid month")
    } else {
        NaiveDate::from_ymd_opt(current.year(), current.month() - 1, 1).expect("valid month")
    };
    DateRange {
        end: last_day(start),
        filename: format!("{}.csv", start.format("%Y-%m")),
        start,
    }
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
    fn parses_month_and_day_date_specs() {
        let month = parse_date_spec("2026-08").expect("valid month");
        assert_eq!(month.start, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        assert_eq!(month.end, NaiveDate::from_ymd_opt(2026, 8, 31).unwrap());

        let day = parse_date_spec("2025-08-03").expect("valid day");
        assert_eq!(day.start, NaiveDate::from_ymd_opt(2025, 8, 3).unwrap());
        assert_eq!(day.end, NaiveDate::from_ymd_opt(2025, 8, 3).unwrap());
    }

    #[test]
    fn rejects_a_range_over_the_configured_limit() {
        let args = vec![
            "--from".to_owned(),
            "2025-01-01".to_owned(),
            "--to".to_owned(),
            "2025-01-02".to_owned(),
            "--max-days".to_owned(),
            "1".to_owned(),
        ];
        assert!(date_range(&args).is_err());
    }
}
