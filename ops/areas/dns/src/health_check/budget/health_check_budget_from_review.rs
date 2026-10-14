use super::*;

use std::collections::BTreeMap;

use crate::dns_record::audit as dns_record_audit;

pub fn summarize_dns_record_reviews_for_budget(items: &[dns_record_audit::DnsRecordReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_health_check_budgets_from_review(items: &[dns_record_audit::DnsRecordReview]) -> Vec<HealthCheckBudget> {
    let mut out: Vec<HealthCheckBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_budget(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_health_check_budget(item: &HealthCheckBudget) -> String {
    dns_record_audit::normalize_dns_record_review_code(&item.code)
}
