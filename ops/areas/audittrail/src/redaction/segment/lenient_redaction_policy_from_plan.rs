use super::*;

use std::collections::BTreeMap;

use crate::query::adaptive_query_manifest as query_adaptive_query_manifest;

pub fn summarize_query_plans_for_policy(items: &[query_adaptive_query_manifest::QueryPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_lenient_redaction_policies_from_plan(
    items: &[query_adaptive_query_manifest::QueryPlan],
) -> Vec<LenientRedactionPolicy> {
    let mut out: Vec<LenientRedactionPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_redaction_policy(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_redaction_policy(item: &LenientRedactionPolicy) -> String {
    query_adaptive_query_manifest::scheduled_query_reservation_bucket(item.projected_volume())
}
