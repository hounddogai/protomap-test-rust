use super::*;

use acme_kit::text;

/// How screens and exports show a lenient rule quota.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleQuotaView {
    pub code: String,
    pub name: String,
    pub host: String,
    pub region: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_lenient_rule_quota_view(item: &LenientRuleQuota) -> LenientRuleQuotaView {
    LenientRuleQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        host: item.host.clone(),
        region: text::truncate(&item.region, 120),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_rule_quota_views(items: &[LenientRuleQuota]) -> Vec<LenientRuleQuotaView> {
    let mut views: Vec<LenientRuleQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_rule_quota_view(item));
    }
    views
}

pub fn sum_lenient_rule_quota_view(view: &LenientRuleQuotaView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_rule_quota("code-847", "name-125");
        let view = to_lenient_rule_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_rule_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
