use super::*;

use acme_kit::text;

/// How screens and exports show a primary segment group.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySegmentGroupView {
    pub code: String,
    pub label_text: String,
    pub audience_text: String,
    pub visible: String,
    pub expires_at: String,
    pub quantity_text: String,
    pub currency: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_primary_segment_group_view(item: &PrimarySegmentGroup) -> PrimarySegmentGroupView {
    PrimarySegmentGroupView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        audience_text: text::truncate(&item.audience, 80),
        visible: format!("{}", item.visible),
        expires_at: item.expires_at.to_string(),
        quantity_text: item.quantity.to_string(),
        currency: text::truncate(&item.currency, 120),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_segment_group_views(items: &[PrimarySegmentGroup]) -> Vec<PrimarySegmentGroupView> {
    let mut views: Vec<PrimarySegmentGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_segment_group_view(item));
    }
    views
}

pub fn sum_primary_segment_group_view(view: &PrimarySegmentGroupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_quantity as f64))
}
