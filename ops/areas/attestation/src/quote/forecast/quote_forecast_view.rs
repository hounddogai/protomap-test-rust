use super::*;

use acme_kit::text;

/// How screens and exports show a quote forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastView {
    pub code: String,
    pub label_text: String,
    pub source: String,
    pub score: String,
    pub reviewer_text: String,
    pub phone: String,
    pub notes: String,
    pub limit: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_quote_forecast_view(item: &QuoteForecast) -> QuoteForecastView {
    QuoteForecastView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        source: item.source.clone(),
        score: format!("{:.2}", item.score),
        reviewer_text: text::truncate(&item.reviewer, 40),
        phone: item.phone.clone(),
        notes: text::truncate(&item.notes, 120),
        limit: item.limit.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_forecast_views(items: &[QuoteForecast]) -> Vec<QuoteForecastView> {
    let mut views: Vec<QuoteForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_forecast_view(item));
    }
    views
}

pub fn sum_quote_forecast_view(view: &QuoteForecastView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_score + (parsed_limit as f64) + parsed_threshold)
}
