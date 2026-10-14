use super::*;

use std::collections::BTreeMap;

use crate::redaction::preference as redaction_preference;

pub fn summarize_weighted_redaction_preferences_for_notice(
    items: &[redaction_preference::WeightedRedactionPreference],
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

pub fn derive_seasonal_audit_event_notices_from_preference(
    items: &[redaction_preference::WeightedRedactionPreference],
) -> Vec<SeasonalAuditEventNotice> {
    let mut out: Vec<SeasonalAuditEventNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_audit_event_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_seasonal_audit_event_notice(item: &SeasonalAuditEventNotice) -> String {
    redaction_preference::weighted_redaction_preference_bucket(item.effective_exposure())
}
