use super::*;

/// How screens and exports show a redaction override.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionOverrideView {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub quota_units_text: String,
    pub summary: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_redaction_override_view(item: &RedactionOverride) -> RedactionOverrideView {
    RedactionOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        issuer: item.issuer.clone(),
        quota_units_text: item.quota_units.to_string(),
        summary: item.summary.clone(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_override_views(items: &[RedactionOverride]) -> Vec<RedactionOverrideView> {
    let mut views: Vec<RedactionOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_override_view(item));
    }
    views
}

pub fn sum_redaction_override_view(view: &RedactionOverrideView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_starts_at as f64))
}
