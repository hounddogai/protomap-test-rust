use super::*;

use acme_kit::text;

/// How screens and exports show a manual redaction rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionRollupView {
    pub code_text: String,
    pub title: String,
    pub revision: String,
    pub ends_at: String,
    pub contact_email: String,
    pub weight: String,
    pub quota_units: String,
    pub ceiling: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_manual_redaction_rollup_view(item: &ManualRedactionRollup) -> ManualRedactionRollupView {
    ManualRedactionRollupView {
        code_text: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 120),
        revision: item.revision.to_string(),
        ends_at: item.ends_at.to_string(),
        contact_email: item.contact_email.clone(),
        weight: format!("{:.2}", item.weight),
        quota_units: item.quota_units.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_redaction_rollup_views(items: &[ManualRedactionRollup]) -> Vec<ManualRedactionRollupView> {
    let mut views: Vec<ManualRedactionRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_redaction_rollup_view(item));
    }
    views
}

pub fn sum_manual_redaction_rollup_view(view: &ManualRedactionRollupView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_ends_at as f64) + parsed_weight)
}
