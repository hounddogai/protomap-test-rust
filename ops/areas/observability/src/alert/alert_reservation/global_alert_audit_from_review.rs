use super::*;

use std::collections::BTreeMap;

use crate::slo::record_item as slo_record_item;

pub fn summarize_slo_reviews_for_audit(items: &[slo_record_item::SloReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_global_alert_audits_from_review(items: &[slo_record_item::SloReview]) -> Vec<GlobalAlertAudit> {
    let mut out: Vec<GlobalAlertAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_global_alert_audit(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_alert_audit(item: &GlobalAlertAudit) -> f64 {
    slo_record_item::slo_review_weight(3, true)
}
