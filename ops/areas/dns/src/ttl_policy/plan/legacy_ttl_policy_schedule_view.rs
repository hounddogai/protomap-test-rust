use super::*;

use acme_kit::text;

/// How screens and exports show a legacy ttl policy schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleView {
    pub code_text: String,
    pub name: String,
    pub country_text: String,
    pub quota_units_text: String,
    pub region: String,
    pub notes: String,
    pub ceiling_text: String,
    pub score: String,
    pub source: String,
    pub principal_text: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_legacy_ttl_policy_schedule_view(item: &LegacyTtlPolicySchedule) -> LegacyTtlPolicyScheduleView {
    LegacyTtlPolicyScheduleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        country_text: item.country.clone(),
        quota_units_text: item.quota_units.to_string(),
        region: item.region.clone(),
        notes: item.notes.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        score: format!("{:.2}", item.score),
        source: text::truncate(&item.source, 120),
        principal_text: item.principal.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_ttl_policy_schedule_views(items: &[LegacyTtlPolicySchedule]) -> Vec<LegacyTtlPolicyScheduleView> {
    let mut views: Vec<LegacyTtlPolicyScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_ttl_policy_schedule_view(item));
    }
    views
}

pub fn sum_legacy_ttl_policy_schedule_view(view: &LegacyTtlPolicyScheduleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ceiling + parsed_score)
}
