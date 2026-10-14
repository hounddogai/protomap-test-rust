use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive forecast score.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastScoreView {
    pub code: String,
    pub name: String,
    pub ratio: String,
    pub timezone: String,
    pub ceiling: String,
    pub cipher_text: String,
    pub locale: String,
    pub scope: String,
    pub amount_text: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_adaptive_forecast_score_view(item: &AdaptiveForecastScore) -> AdaptiveForecastScoreView {
    AdaptiveForecastScoreView {
        code: item.code.clone(),
        name: item.name.clone(),
        ratio: format!("{:.2}", item.ratio),
        timezone: text::truncate(&item.timezone, 40),
        ceiling: format!("{:.2}", item.ceiling),
        cipher_text: item.cipher.clone(),
        locale: item.locale.clone(),
        scope: item.scope.clone(),
        amount_text: format!("{:.2}", item.amount),
        issuer_text: text::truncate(&item.issuer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_forecast_score_views(items: &[AdaptiveForecastScore]) -> Vec<AdaptiveForecastScoreView> {
    let mut views: Vec<AdaptiveForecastScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_forecast_score_view(item));
    }
    views
}

pub fn sum_adaptive_forecast_score_view(view: &AdaptiveForecastScoreView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ratio + parsed_ceiling + parsed_amount)
}
