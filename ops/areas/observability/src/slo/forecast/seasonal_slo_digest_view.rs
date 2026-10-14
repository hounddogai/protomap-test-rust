use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal slo digest.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigestView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub quota_units: String,
    pub fingerprint: String,
    pub category: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_seasonal_slo_digest_view(item: &SeasonalSloDigest) -> SeasonalSloDigestView {
    SeasonalSloDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        quota_units: item.quota_units.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        category: item.category.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_slo_digest_views(items: &[SeasonalSloDigest]) -> Vec<SeasonalSloDigestView> {
    let mut views: Vec<SeasonalSloDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_slo_digest_view(item));
    }
    views
}

pub fn sum_seasonal_slo_digest_view(view: &SeasonalSloDigestView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_created_at as f64))
}
