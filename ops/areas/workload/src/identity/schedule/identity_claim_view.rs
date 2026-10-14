use super::*;

use acme_kit::text;

/// How screens and exports show a identity claim.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaimView {
    pub code: String,
    pub title_text: String,
    pub quota_units_text: String,
    pub region: String,
    pub principal: String,
    pub reviewer_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_identity_claim_view(item: &IdentityClaim) -> IdentityClaimView {
    IdentityClaimView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quota_units_text: item.quota_units.to_string(),
        region: item.region.clone(),
        principal: text::truncate(&item.principal, 64),
        reviewer_text: item.reviewer.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_claim_views(items: &[IdentityClaim]) -> Vec<IdentityClaimView> {
    let mut views: Vec<IdentityClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_claim_view(item));
    }
    views
}

pub fn sum_identity_claim_view(view: &IdentityClaimView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_claim("code-739", "title-660");
        let view = to_identity_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
