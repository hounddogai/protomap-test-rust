use super::*;

use acme_kit::text;

/// How screens and exports show a credential forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecastView {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub duration_minutes: String,
    pub audience_text: String,
    pub notes: String,
    pub summary: String,
    pub minimum: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_credential_forecast_view(item: &CredentialForecast) -> CredentialForecastView {
    CredentialForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        channel: item.channel.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        audience_text: item.audience.clone(),
        notes: item.notes.clone(),
        summary: text::truncate(&item.summary, 40),
        minimum: format!("{:.2}", item.minimum),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_forecast_views(items: &[CredentialForecast]) -> Vec<CredentialForecastView> {
    let mut views: Vec<CredentialForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_forecast_view(item));
    }
    views
}

pub fn sum_credential_forecast_view(view: &CredentialForecastView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_minimum + parsed_ceiling)
}
