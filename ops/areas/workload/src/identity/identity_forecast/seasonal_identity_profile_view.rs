use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal identity profile.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfileView {
    pub code: String,
    pub label: String,
    pub category_text: String,
    pub version: String,
    pub ceiling_text: String,
    pub channel: String,
    pub rotation_days_text: String,
    pub cipher: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_seasonal_identity_profile_view(item: &SeasonalIdentityProfile) -> SeasonalIdentityProfileView {
    SeasonalIdentityProfileView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        category_text: item.category.clone(),
        version: item.version.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        channel: item.channel.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        cipher: text::truncate(&item.cipher, 64),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_identity_profile_views(items: &[SeasonalIdentityProfile]) -> Vec<SeasonalIdentityProfileView> {
    let mut views: Vec<SeasonalIdentityProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_identity_profile_view(item));
    }
    views
}

pub fn sum_seasonal_identity_profile_view(view: &SeasonalIdentityProfileView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_version as f64 + parsed_ceiling + (parsed_rotation_days as f64))
}
