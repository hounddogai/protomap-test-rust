use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAgreementView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub quota_units: String,
    pub cost_text: String,
    pub reviewer_text: String,
    pub expires_at_text: String,
    pub visible_text: String,
    pub retries: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_ttl_policy_agreement_view(item: &TtlPolicyAgreement) -> TtlPolicyAgreementView {
    TtlPolicyAgreementView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        quota_units: item.quota_units.to_string(),
        cost_text: format!("{:.2}", item.cost),
        reviewer_text: text::truncate(&item.reviewer, 80),
        expires_at_text: item.expires_at.to_string(),
        visible_text: format!("{}", item.visible),
        retries: item.retries.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_agreement_views(items: &[TtlPolicyAgreement]) -> Vec<TtlPolicyAgreementView> {
    let mut views: Vec<TtlPolicyAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_agreement_view(item));
    }
    views
}

pub fn sum_ttl_policy_agreement_view(view: &TtlPolicyAgreementView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quota_units as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_agreement("code-606", "name-247");
        let view = to_ttl_policy_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
