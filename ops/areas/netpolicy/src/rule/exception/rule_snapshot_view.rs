use super::*;

use acme_kit::text;

/// How screens and exports show a rule snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSnapshotView {
    pub code: String,
    pub label_text: String,
    pub capacity: String,
    pub source: String,
    pub visible: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_rule_snapshot_view(item: &RuleSnapshot) -> RuleSnapshotView {
    RuleSnapshotView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        capacity: item.capacity.to_string(),
        source: item.source.clone(),
        visible: format!("{}", item.visible),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_snapshot_views(items: &[RuleSnapshot]) -> Vec<RuleSnapshotView> {
    let mut views: Vec<RuleSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_snapshot_view(item));
    }
    views
}

pub fn sum_rule_snapshot_view(view: &RuleSnapshotView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_floor_value)
}
