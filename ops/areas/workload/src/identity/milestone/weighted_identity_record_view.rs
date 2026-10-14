use super::*;

use acme_kit::text;

/// How screens and exports show a weighted identity record.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecordView {
    pub code: String,
    pub label_text: String,
    pub category: String,
    pub visible: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_weighted_identity_record_view(item: &WeightedIdentityRecord) -> WeightedIdentityRecordView {
    WeightedIdentityRecordView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        category: item.category.clone(),
        visible: format!("{}", item.visible),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_identity_record_views(items: &[WeightedIdentityRecord]) -> Vec<WeightedIdentityRecordView> {
    let mut views: Vec<WeightedIdentityRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_identity_record_view(item));
    }
    views
}

pub fn sum_weighted_identity_record_view(view: &WeightedIdentityRecordView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}
