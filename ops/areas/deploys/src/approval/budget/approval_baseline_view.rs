use super::*;

use acme_kit::text;

/// How screens and exports show a approval baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBaselineView {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub country_text: String,
    pub key_id: String,
    pub version: String,
    pub notes: String,
    pub minimum_text: String,
    pub channel: String,
    pub sequence: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_approval_baseline_view(item: &ApprovalBaseline) -> ApprovalBaselineView {
    ApprovalBaselineView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        phone: text::truncate(&item.phone, 64),
        country_text: item.country.clone(),
        key_id: text::truncate(&item.key_id, 80),
        version: item.version.to_string(),
        notes: item.notes.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        channel: text::truncate(&item.channel, 80),
        sequence: item.sequence.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_baseline_views(items: &[ApprovalBaseline]) -> Vec<ApprovalBaselineView> {
    let mut views: Vec<ApprovalBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_baseline_view(item));
    }
    views
}

pub fn sum_approval_baseline_view(view: &ApprovalBaselineView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_version as f64 + parsed_minimum + (parsed_sequence as f64))
}
