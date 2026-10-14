use super::*;

use std::collections::BTreeMap;

use crate::token::token_schedule as token_token_schedule;

pub fn summarize_rolling_token_schedules_for_score(
    items: &[token_token_schedule::RollingTokenSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_capped_trust_domain_scores_from_schedule(
    items: &[token_token_schedule::RollingTokenSchedule],
) -> Vec<CappedTrustDomainScore> {
    let mut out: Vec<CappedTrustDomainScore> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_trust_domain_score(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_capped_trust_domain_score(item: &CappedTrustDomainScore) -> f64 {
    token_token_schedule::tiered_token_segment_weight(8, true)
}
