use super::*;

use acme_kit::text;

/// How screens and exports show a rolling ttl policy agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementView {
    pub code: String,
    pub name: String,
    pub rotation_days: String,
    pub notes: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_rolling_ttl_policy_agreement_view(item: &RollingTtlPolicyAgreement) -> RollingTtlPolicyAgreementView {
    RollingTtlPolicyAgreementView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        rotation_days: item.rotation_days.to_string(),
        notes: text::truncate(&item.notes, 120),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_ttl_policy_agreement_views(
    items: &[RollingTtlPolicyAgreement],
) -> Vec<RollingTtlPolicyAgreementView> {
    let mut views: Vec<RollingTtlPolicyAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_ttl_policy_agreement_view(item));
    }
    views
}

pub fn sum_rolling_ttl_policy_agreement_view(view: &RollingTtlPolicyAgreementView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_ttl_policy_agreement("code-914", "name-791");
        let view = to_rolling_ttl_policy_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_ttl_policy_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
