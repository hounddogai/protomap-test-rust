use super::*;

use std::collections::BTreeMap;

use crate::redaction::adjustment as redaction_adjustment;

pub fn summarize_regional_redaction_groups_for_digest(
    items: &[redaction_adjustment::RegionalRedactionGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_scheduled_sink_digests_from_group(
    items: &[redaction_adjustment::RegionalRedactionGroup],
) -> Vec<ScheduledSinkDigest> {
    let mut out: Vec<ScheduledSinkDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_sink_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_sink_digest(item: &ScheduledSinkDigest) -> String {
    redaction_adjustment::primary_redaction_template_bucket(item.adjusted_exposure())
}
