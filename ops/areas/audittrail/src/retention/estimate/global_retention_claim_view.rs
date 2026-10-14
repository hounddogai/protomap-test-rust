use super::*;

use acme_kit::text;

/// How screens and exports show a global retention claim.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionClaimView {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub description: String,
    pub issuer_text: String,
    pub expires_at: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_global_retention_claim_view(item: &GlobalRetentionClaim) -> GlobalRetentionClaimView {
    GlobalRetentionClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        phone: item.phone.clone(),
        description: item.description.clone(),
        issuer_text: text::truncate(&item.issuer, 120),
        expires_at: item.expires_at.to_string(),
        audience: text::truncate(&item.audience, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_retention_claim_views(items: &[GlobalRetentionClaim]) -> Vec<GlobalRetentionClaimView> {
    let mut views: Vec<GlobalRetentionClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_retention_claim_view(item));
    }
    views
}

pub fn sum_global_retention_claim_view(view: &GlobalRetentionClaimView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_retention_claim("code-300", "label-75");
        let view = to_global_retention_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_retention_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
