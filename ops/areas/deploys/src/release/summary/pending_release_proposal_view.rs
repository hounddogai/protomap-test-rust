use super::*;

/// How screens and exports show a pending release proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalView {
    pub code_text: String,
    pub label: String,
    pub timezone: String,
    pub unit: String,
    pub phone_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_pending_release_proposal_view(item: &PendingReleaseProposal) -> PendingReleaseProposalView {
    PendingReleaseProposalView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        unit: item.unit.clone(),
        phone_text: item.phone.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_release_proposal_views(items: &[PendingReleaseProposal]) -> Vec<PendingReleaseProposalView> {
    let mut views: Vec<PendingReleaseProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_release_proposal_view(item));
    }
    views
}

pub fn sum_pending_release_proposal_view(view: &PendingReleaseProposalView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_release_proposal("code-394", "label-314");
        let view = to_pending_release_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_release_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
