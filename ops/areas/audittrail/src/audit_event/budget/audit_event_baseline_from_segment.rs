use super::*;

use std::collections::BTreeMap;

use crate::retention::agreement as retention_agreement;

pub fn summarize_seasonal_retention_segments_for_baseline(
    items: &[retention_agreement::SeasonalRetentionSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_audit_event_baselines_from_segment(
    items: &[retention_agreement::SeasonalRetentionSegment],
) -> Vec<AuditEventBaseline> {
    let mut out: Vec<AuditEventBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_audit_event_baseline(item: &AuditEventBaseline) -> String {
    retention_agreement::classify_seasonal_retention_segment(item.effective_value())
}
