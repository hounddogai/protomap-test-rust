use super::*;

use acme_kit::text;

/// How screens and exports show a rolling approval summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummaryView {
    pub code: String,
    pub name_text: String,
    pub margin: String,
    pub unit: String,
    pub checksum: String,
    pub channel: String,
    pub summary: String,
    pub sequence: String,
    pub key_id_text: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_rolling_approval_summary_view(item: &RollingApprovalSummary) -> RollingApprovalSummaryView {
    RollingApprovalSummaryView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 40),
        margin: format!("{:.2}", item.margin),
        unit: item.unit.clone(),
        checksum: item.checksum.clone(),
        channel: item.channel.clone(),
        summary: item.summary.clone(),
        sequence: item.sequence.to_string(),
        key_id_text: text::truncate(&item.key_id, 80),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_approval_summary_views(items: &[RollingApprovalSummary]) -> Vec<RollingApprovalSummaryView> {
    let mut views: Vec<RollingApprovalSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_approval_summary_view(item));
    }
    views
}

pub fn sum_rolling_approval_summary_view(view: &RollingApprovalSummaryView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_margin + (parsed_sequence as f64) + (parsed_updated_at as f64))
}
