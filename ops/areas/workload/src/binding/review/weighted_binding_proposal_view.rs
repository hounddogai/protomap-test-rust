use super::*;

use acme_kit::text;

/// How screens and exports show a weighted binding proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBindingProposalView {
    pub code: String,
    pub title_text: String,
    pub priority: String,
    pub ttl_seconds: String,
    pub pinned: String,
    pub visible: String,
    pub audience: String,
    pub expires_at: String,
    pub quota_units_text: String,
    pub category: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_weighted_binding_proposal_view(item: &WeightedBindingProposal) -> WeightedBindingProposalView {
    WeightedBindingProposalView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        priority: item.priority.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        pinned: format!("{}", item.pinned),
        visible: format!("{}", item.visible),
        audience: text::truncate(&item.audience, 40),
        expires_at: item.expires_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        category: item.category.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_binding_proposal_views(items: &[WeightedBindingProposal]) -> Vec<WeightedBindingProposalView> {
    let mut views: Vec<WeightedBindingProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_binding_proposal_view(item));
    }
    views
}

pub fn sum_weighted_binding_proposal_view(view: &WeightedBindingProposalView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_ttl_seconds as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_binding_proposal("code-628", "title-295");
        let view = to_weighted_binding_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_binding_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
