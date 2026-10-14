use super::*;

use acme_kit::text;

/// How screens and exports show a regional freeze snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeSnapshotView {
    pub code_text: String,
    pub name: String,
    pub currency_text: String,
    pub key_id: String,
    pub threshold: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_regional_freeze_snapshot_view(item: &RegionalFreezeSnapshot) -> RegionalFreezeSnapshotView {
    RegionalFreezeSnapshotView {
        code_text: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        currency_text: item.currency.clone(),
        key_id: item.key_id.clone(),
        threshold: format!("{:.2}", item.threshold),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_freeze_snapshot_views(items: &[RegionalFreezeSnapshot]) -> Vec<RegionalFreezeSnapshotView> {
    let mut views: Vec<RegionalFreezeSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_freeze_snapshot_view(item));
    }
    views
}

pub fn sum_regional_freeze_snapshot_view(view: &RegionalFreezeSnapshotView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}
