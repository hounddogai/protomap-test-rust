use super::*;

/// How screens and exports show a evidence forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecastView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub score_text: String,
    pub namespace: String,
    pub visible: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_evidence_forecast_view(item: &EvidenceForecast) -> EvidenceForecastView {
    EvidenceForecastView {
        code: item.code.clone(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        score_text: format!("{:.2}", item.score),
        namespace: item.namespace.clone(),
        visible: format!("{}", item.visible),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_forecast_views(items: &[EvidenceForecast]) -> Vec<EvidenceForecastView> {
    let mut views: Vec<EvidenceForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_forecast_view(item));
    }
    views
}

pub fn sum_evidence_forecast_view(view: &EvidenceForecastView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_amount + parsed_score + (parsed_starts_at as f64))
}
