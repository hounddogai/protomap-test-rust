use super::*;

use acme_kit::text;

/// How screens and exports show a primary trust domain target.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTargetView {
    pub code: String,
    pub label_text: String,
    pub category: String,
    pub locale: String,
    pub cipher: String,
    pub contact_email: String,
    pub owner_id: String,
    pub external_ref_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_primary_trust_domain_target_view(item: &PrimaryTrustDomainTarget) -> PrimaryTrustDomainTargetView {
    PrimaryTrustDomainTargetView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        category: item.category.clone(),
        locale: text::truncate(&item.locale, 64),
        cipher: item.cipher.clone(),
        contact_email: item.contact_email.clone(),
        owner_id: item.owner_id.clone(),
        external_ref_text: item.external_ref.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_trust_domain_target_views(items: &[PrimaryTrustDomainTarget]) -> Vec<PrimaryTrustDomainTargetView> {
    let mut views: Vec<PrimaryTrustDomainTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_trust_domain_target_view(item));
    }
    views
}

pub fn sum_primary_trust_domain_target_view(view: &PrimaryTrustDomainTargetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_trust_domain_target("code-62", "label-693");
        let view = to_primary_trust_domain_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_trust_domain_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
