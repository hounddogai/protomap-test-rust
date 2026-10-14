use super::*;

use acme_kit::text;

/// How screens and exports show a legacy redaction rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionRollupView {
    pub code: String,
    pub name: String,
    pub limit_text: String,
    pub quota_units: String,
    pub cost: String,
    pub contact_email: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_legacy_redaction_rollup_view(item: &LegacyRedactionRollup) -> LegacyRedactionRollupView {
    LegacyRedactionRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        limit_text: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        cost: format!("{:.2}", item.cost),
        contact_email: text::truncate(&item.contact_email, 120),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_redaction_rollup_views(items: &[LegacyRedactionRollup]) -> Vec<LegacyRedactionRollupView> {
    let mut views: Vec<LegacyRedactionRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_redaction_rollup_view(item));
    }
    views
}

pub fn sum_legacy_redaction_rollup_view(view: &LegacyRedactionRollupView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_quota_units as f64) + parsed_cost)
}
