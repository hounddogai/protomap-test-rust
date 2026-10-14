use super::*;

use acme_kit::text;

/// How screens and exports show a rollback proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProposalView {
    pub code: String,
    pub title_text: String,
    pub ttl_seconds: String,
    pub owner_id: String,
    pub phone: String,
    pub external_ref: String,
    pub currency: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_rollback_proposal_view(item: &RollbackProposal) -> RollbackProposalView {
    RollbackProposalView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        owner_id: item.owner_id.clone(),
        phone: item.phone.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        currency: text::truncate(&item.currency, 120),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_proposal_views(items: &[RollbackProposal]) -> Vec<RollbackProposalView> {
    let mut views: Vec<RollbackProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_proposal_view(item));
    }
    views
}

pub fn sum_rollback_proposal_view(view: &RollbackProposalView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_proposal("code-129", "title-895");
        let view = to_rollback_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
