use super::*;

use std::collections::BTreeMap;

use crate::dns_record::draft as dns_record_draft;

pub fn summarize_dns_record_drafts_for_review(items: &[dns_record_draft::DnsRecordDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_default_zone_reviews_from_draft(items: &[dns_record_draft::DnsRecordDraft]) -> Vec<DefaultZoneReview> {
    let mut out: Vec<DefaultZoneReview> = Vec::new();
    for item in items.iter() {
        let derived = new_default_zone_review(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_default_zone_review(item: &DefaultZoneReview) -> f64 {
    dns_record_draft::manual_dns_record_adjustment_weight(7, true)
}
