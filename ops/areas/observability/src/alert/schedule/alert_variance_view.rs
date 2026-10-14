use super::*;

use acme_kit::text;

/// How screens and exports show a alert variance.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertVarianceView {
    pub code_text: String,
    pub name: String,
    pub maximum: String,
    pub score: String,
    pub title_text: String,
    pub contact_email: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_alert_variance_view(item: &AlertVariance) -> AlertVarianceView {
    AlertVarianceView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        maximum: format!("{:.2}", item.maximum),
        score: format!("{:.2}", item.score),
        title_text: item.title.clone(),
        contact_email: item.contact_email.clone(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_variance_views(items: &[AlertVariance]) -> Vec<AlertVarianceView> {
    let mut views: Vec<AlertVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_variance_view(item));
    }
    views
}

pub fn sum_alert_variance_view(view: &AlertVarianceView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_maximum + parsed_score)
}
