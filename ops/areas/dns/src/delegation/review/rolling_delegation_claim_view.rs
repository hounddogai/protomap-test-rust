use super::*;

use acme_kit::text;

/// How screens and exports show a rolling delegation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaimView {
    pub code_text: String,
    pub title: String,
    pub source: String,
    pub category: String,
    pub key_id: String,
    pub grace_days: String,
    pub description: String,
    pub created_at_text: String,
    pub issuer_text: String,
    pub phone: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_rolling_delegation_claim_view(item: &RollingDelegationClaim) -> RollingDelegationClaimView {
    RollingDelegationClaimView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        source: text::truncate(&item.source, 80),
        category: item.category.clone(),
        key_id: text::truncate(&item.key_id, 120),
        grace_days: item.grace_days.to_string(),
        description: item.description.clone(),
        created_at_text: item.created_at.to_string(),
        issuer_text: item.issuer.clone(),
        phone: item.phone.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_delegation_claim_views(items: &[RollingDelegationClaim]) -> Vec<RollingDelegationClaimView> {
    let mut views: Vec<RollingDelegationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_delegation_claim_view(item));
    }
    views
}

pub fn sum_rolling_delegation_claim_view(view: &RollingDelegationClaimView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_created_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_delegation_claim("code-913", "title-469");
        let view = to_rolling_delegation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_delegation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
