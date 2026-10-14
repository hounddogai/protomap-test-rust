use super::*;

/// How screens and exports show a strict quote baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQuoteBaselineView {
    pub code: String,
    pub title: String,
    pub margin_text: String,
    pub ttl_seconds: String,
    pub phone_text: String,
    pub discount: String,
    pub ends_at: String,
    pub weight: String,
    pub cost_text: String,
    pub description: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_strict_quote_baseline_view(item: &StrictQuoteBaseline) -> StrictQuoteBaselineView {
    StrictQuoteBaselineView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin_text: format!("{:.2}", item.margin),
        ttl_seconds: item.ttl_seconds.to_string(),
        phone_text: item.phone.clone(),
        discount: format!("{:.2}", item.discount),
        ends_at: item.ends_at.to_string(),
        weight: format!("{:.2}", item.weight),
        cost_text: format!("{:.2}", item.cost),
        description: item.description.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_quote_baseline_views(items: &[StrictQuoteBaseline]) -> Vec<StrictQuoteBaselineView> {
    let mut views: Vec<StrictQuoteBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_quote_baseline_view(item));
    }
    views
}

pub fn sum_strict_quote_baseline_view(view: &StrictQuoteBaselineView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_margin + (parsed_ttl_seconds as f64) + parsed_discount)
}
