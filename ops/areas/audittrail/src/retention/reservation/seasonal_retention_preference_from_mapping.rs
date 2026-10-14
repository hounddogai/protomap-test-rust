use super::*;

use std::collections::BTreeMap;

use crate::audit_event::milestone as audit_event_milestone;

pub fn summarize_audit_event_mappings_for_preference(
    items: &[audit_event_milestone::AuditEventMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_seasonal_retention_preferences_from_mapping(
    items: &[audit_event_milestone::AuditEventMapping],
) -> Vec<SeasonalRetentionPreference> {
    let mut out: Vec<SeasonalRetentionPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_retention_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_seasonal_retention_preference(item: &SeasonalRetentionPreference) -> String {
    audit_event_milestone::audit_event_milestone_bucket(item.projected_value())
}
