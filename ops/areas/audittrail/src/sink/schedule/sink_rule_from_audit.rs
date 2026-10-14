use super::*;

use std::collections::BTreeMap;

use crate::query::profile as query_profile;

pub fn summarize_capped_query_audits_for_rule(items: &[query_profile::CappedQueryAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_sink_rules_from_audit(items: &[query_profile::CappedQueryAudit]) -> Vec<SinkRule> {
    let mut out: Vec<SinkRule> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_sink_rule(item: &SinkRule) -> String {
    query_profile::classify_seasonal_query_variance(item.weighted_value())
}
