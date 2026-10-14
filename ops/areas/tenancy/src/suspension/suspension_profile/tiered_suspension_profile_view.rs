use super::*;

use acme_kit::text;

/// How screens and exports show a tiered suspension profile.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionProfileView {
    pub code: String,
    pub name_text: String,
    pub summary: String,
    pub rotation_days: String,
    pub rate: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_tiered_suspension_profile_view(item: &TieredSuspensionProfile) -> TieredSuspensionProfileView {
    TieredSuspensionProfileView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        summary: item.summary.clone(),
        rotation_days: item.rotation_days.to_string(),
        rate: format!("{:.2}", item.rate),
        tenant_id: text::truncate(&item.tenant_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_suspension_profile_views(items: &[TieredSuspensionProfile]) -> Vec<TieredSuspensionProfileView> {
    let mut views: Vec<TieredSuspensionProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_suspension_profile_view(item));
    }
    views
}

pub fn sum_tiered_suspension_profile_view(view: &TieredSuspensionProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_rate)
}
