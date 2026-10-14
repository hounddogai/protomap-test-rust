use super::*;

use std::collections::BTreeMap;

use crate::retention::override_item as retention_override_item;

pub fn summarize_retention_estimates_for_template(
    items: &[retention_override_item::RetentionEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_redaction_templates_from_estimate(
    items: &[retention_override_item::RetentionEstimate],
) -> Vec<RedactionTemplate> {
    let mut out: Vec<RedactionTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_template(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_redaction_template(item: &RedactionTemplate) -> String {
    retention_override_item::classify_batched_retention_override(item.measured_value())
}
