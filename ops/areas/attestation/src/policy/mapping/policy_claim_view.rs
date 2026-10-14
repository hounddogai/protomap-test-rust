use super::*;

use acme_kit::text;

/// How screens and exports show a policy claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaimView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub scope: String,
    pub timezone: String,
    pub external_ref: String,
    pub audience: String,
    pub segment: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_policy_claim_view(item: &PolicyClaim) -> PolicyClaimView {
    PolicyClaimView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        cluster: item.cluster.clone(),
        scope: item.scope.clone(),
        timezone: text::truncate(&item.timezone, 120),
        external_ref: item.external_ref.clone(),
        audience: item.audience.clone(),
        segment: item.segment.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_claim_views(items: &[PolicyClaim]) -> Vec<PolicyClaimView> {
    let mut views: Vec<PolicyClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_claim_view(item));
    }
    views
}

pub fn sum_policy_claim_view(view: &PolicyClaimView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_claim("code-849", "name-665");
        let view = to_policy_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
