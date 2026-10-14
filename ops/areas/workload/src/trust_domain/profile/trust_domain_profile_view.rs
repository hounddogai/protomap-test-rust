use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain profile.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileView {
    pub code: String,
    pub title_text: String,
    pub priority: String,
    pub sequence_text: String,
    pub created_at: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_trust_domain_profile_view(item: &TrustDomainProfile) -> TrustDomainProfileView {
    TrustDomainProfileView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        priority: item.priority.to_string(),
        sequence_text: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        external_ref_text: text::truncate(&item.external_ref, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_profile_views(items: &[TrustDomainProfile]) -> Vec<TrustDomainProfileView> {
    let mut views: Vec<TrustDomainProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_profile_view(item));
    }
    views
}

pub fn sum_trust_domain_profile_view(view: &TrustDomainProfileView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_sequence as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_profile("code-959", "title-441");
        let view = to_trust_domain_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
