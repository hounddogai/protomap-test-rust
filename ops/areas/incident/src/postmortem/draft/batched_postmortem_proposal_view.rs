use super::*;

use acme_kit::text;

/// How screens and exports show a batched postmortem proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProposalView {
    pub code: String,
    pub label: String,
    pub created_at: String,
    pub revision_text: String,
    pub minimum: String,
    pub starts_at: String,
    pub weight: String,
    pub ends_at: String,
    pub title: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_batched_postmortem_proposal_view(item: &BatchedPostmortemProposal) -> BatchedPostmortemProposalView {
    BatchedPostmortemProposalView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        created_at: item.created_at.to_string(),
        revision_text: item.revision.to_string(),
        minimum: format!("{:.2}", item.minimum),
        starts_at: item.starts_at.to_string(),
        weight: format!("{:.2}", item.weight),
        ends_at: item.ends_at.to_string(),
        title: item.title.clone(),
        phone: text::truncate(&item.phone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_postmortem_proposal_views(items: &[BatchedPostmortemProposal]) -> Vec<BatchedPostmortemProposalView> {
    let mut views: Vec<BatchedPostmortemProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_postmortem_proposal_view(item));
    }
    views
}

pub fn sum_batched_postmortem_proposal_view(view: &BatchedPostmortemProposalView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_revision as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_postmortem_proposal("code-176", "label-559");
        let view = to_batched_postmortem_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_postmortem_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
