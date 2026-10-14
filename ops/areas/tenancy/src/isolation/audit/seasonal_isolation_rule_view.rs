use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal isolation rule.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationRuleView {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub updated_at: String,
    pub attempts: String,
    pub phone: String,
    pub cipher: String,
    pub quantity_text: String,
    pub weight: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_seasonal_isolation_rule_view(item: &SeasonalIsolationRule) -> SeasonalIsolationRuleView {
    SeasonalIsolationRuleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        issuer: item.issuer.clone(),
        updated_at: item.updated_at.to_string(),
        attempts: item.attempts.to_string(),
        phone: text::truncate(&item.phone, 120),
        cipher: item.cipher.clone(),
        quantity_text: item.quantity.to_string(),
        weight: format!("{:.2}", item.weight),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_isolation_rule_views(items: &[SeasonalIsolationRule]) -> Vec<SeasonalIsolationRuleView> {
    let mut views: Vec<SeasonalIsolationRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_isolation_rule_view(item));
    }
    views
}

pub fn sum_seasonal_isolation_rule_view(view: &SeasonalIsolationRuleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_attempts as f64) + (parsed_quantity as f64))
}
