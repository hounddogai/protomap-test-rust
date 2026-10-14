use super::*;

use acme_kit::text;

/// How screens and exports show a verifier claim.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimView {
    pub code: String,
    pub title_text: String,
    pub tenant_id_text: String,
    pub unit: String,
    pub priority: String,
    pub ends_at: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_verifier_claim_view(item: &VerifierClaim) -> VerifierClaimView {
    VerifierClaimView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 40),
        unit: item.unit.clone(),
        priority: item.priority.to_string(),
        ends_at: item.ends_at.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_claim_views(items: &[VerifierClaim]) -> Vec<VerifierClaimView> {
    let mut views: Vec<VerifierClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_claim_view(item));
    }
    views
}

pub fn sum_verifier_claim_view(view: &VerifierClaimView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_claim("code-529", "title-461");
        let view = to_verifier_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
