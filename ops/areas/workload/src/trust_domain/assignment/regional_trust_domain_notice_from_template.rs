use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::template as trust_domain_template;

pub fn summarize_primary_trust_domain_templates_for_notice(
    items: &[trust_domain_template::PrimaryTrustDomainTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_regional_trust_domain_notices_from_template(
    items: &[trust_domain_template::PrimaryTrustDomainTemplate],
) -> Vec<RegionalTrustDomainNotice> {
    let mut out: Vec<RegionalTrustDomainNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_trust_domain_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_regional_trust_domain_notice(item: &RegionalTrustDomainNotice) -> String {
    trust_domain_template::capped_trust_domain_score_bucket(item.projected_load())
}
