use super::*;

use std::collections::BTreeMap;

use crate::dns_record::policy as dns_record_policy;

pub fn summarize_dns_record_scores_for_plan(items: &[dns_record_policy::DnsRecordScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_primary_ttl_policy_plans_from_score(
    items: &[dns_record_policy::DnsRecordScore],
) -> Vec<PrimaryTtlPolicyPlan> {
    let mut out: Vec<PrimaryTtlPolicyPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_ttl_policy_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_ttl_policy_plan(item: &PrimaryTtlPolicyPlan) -> String {
    dns_record_policy::dns_record_policy_bucket(item.projected_volume())
}
