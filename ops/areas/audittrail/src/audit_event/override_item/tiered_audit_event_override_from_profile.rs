use super::*;

use std::collections::BTreeMap;

use crate::retention::rule as retention_rule;

pub fn summarize_strict_retention_profiles_for_override(
    items: &[retention_rule::StrictRetentionProfile],
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

pub fn derive_tiered_audit_event_overrides_from_profile(
    items: &[retention_rule::StrictRetentionProfile],
) -> Vec<TieredAuditEventOverride> {
    let mut out: Vec<TieredAuditEventOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_audit_event_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_tiered_audit_event_override(item: &TieredAuditEventOverride) -> String {
    retention_rule::strict_retention_profile_bucket(item.net_total())
}
