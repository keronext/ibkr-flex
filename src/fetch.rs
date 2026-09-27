use crate::Result;
use chrono::NaiveDate;
use std::time::Duration;

const BASE: &str = "https://ndcdyn.interactivebrokers.com/AccountManagement/FlexWebService";

pub async fn download_report(
    token: &str,
    query_id: &str,
    date_start: NaiveDate,
    date_end: NaiveDate,
) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("ibkr-flex-cli/0.1")
        .build()
        .map_err(|error| error.to_string())?;
    let from = date_start.format("%Y%m%d").to_string();
    let to = date_end.format("%Y%m%d").to_string();
    let response = client
        .get(format!("{BASE}/SendRequest"))
        .query(&[
            ("t", token),
            ("q", query_id),
            ("v", "3"),
            ("fd", &from),
            ("td", &to),
        ])
        .send()
        .await
        .map_err(|error| error.to_string())?
        .text()
        .await
        .map_err(|error| error.to_string())?;
    let reference = xml(&response, "ReferenceCode")
        .filter(|_| response.contains("<Status>Success</Status>"))
        .ok_or_else(|| {
            format!(
                "IBKR SendRequest : {}",
                xml(&response, "ErrorMessage").unwrap_or("réponse invalide")
            )
        })?;

    for _ in 0..5 {
        let report = client
            .get(format!("{BASE}/GetStatement"))
            .query(&[("t", token), ("q", reference), ("v", "3")])
            .send()
            .await
            .map_err(|error| error.to_string())?
            .text()
            .await
            .map_err(|error| error.to_string())?;
        if !report.trim_start().starts_with('<') {
            return Ok(report);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err("IBKR n'a pas généré le rapport dans le délai imparti.".to_owned())
}

fn xml<'a>(value: &'a str, element: &str) -> Option<&'a str> {
    value
        .split_once(&format!("<{element}>"))?
        .1
        .split_once(&format!("</{element}>"))
        .map(|(value, _)| value)
}
