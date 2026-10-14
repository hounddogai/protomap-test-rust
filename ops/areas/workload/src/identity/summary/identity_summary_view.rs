use super::*;

/// How screens and exports show a identity summary.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummaryView {
    pub code: String,
    pub label: String,
    pub rate: String,
    pub rotation_days: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_identity_summary_view(item: &IdentitySummary) -> IdentitySummaryView {
    IdentitySummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        rotation_days: item.rotation_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_summary_views(items: &[IdentitySummary]) -> Vec<IdentitySummaryView> {
    let mut views: Vec<IdentitySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_summary_view(item));
    }
    views
}

pub fn sum_identity_summary_view(view: &IdentitySummaryView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_rate + (parsed_rotation_days as f64) + parsed_minimum)
}
