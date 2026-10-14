use super::*;

use acme_kit::text;

/// How screens and exports show a lenient region pin tier.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinTierView {
    pub code_text: String,
    pub label: String,
    pub timezone: String,
    pub quota_units: String,
    pub active: String,
    pub cipher: String,
    pub external_ref: String,
    pub grace_days_text: String,
    pub visible: String,
    pub namespace: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_lenient_region_pin_tier_view(item: &LenientRegionPinTier) -> LenientRegionPinTierView {
    LenientRegionPinTierView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        quota_units: item.quota_units.to_string(),
        active: format!("{}", item.active),
        cipher: text::truncate(&item.cipher, 64),
        external_ref: item.external_ref.clone(),
        grace_days_text: item.grace_days.to_string(),
        visible: format!("{}", item.visible),
        namespace: item.namespace.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_region_pin_tier_views(items: &[LenientRegionPinTier]) -> Vec<LenientRegionPinTierView> {
    let mut views: Vec<LenientRegionPinTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_region_pin_tier_view(item));
    }
    views
}

pub fn sum_lenient_region_pin_tier_view(view: &LenientRegionPinTierView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_grace_days as f64) + (parsed_rotation_days as f64))
}
