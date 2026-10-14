use super::*;

/// How screens and exports show a rollout proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProposalView {
    pub code_text: String,
    pub name: String,
    pub contact_email: String,
    pub unit: String,
    pub margin_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_rollout_proposal_view(item: &RolloutProposal) -> RolloutProposalView {
    RolloutProposalView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        contact_email: item.contact_email.clone(),
        unit: item.unit.clone(),
        margin_text: format!("{:.2}", item.margin),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_proposal_views(items: &[RolloutProposal]) -> Vec<RolloutProposalView> {
    let mut views: Vec<RolloutProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_proposal_view(item));
    }
    views
}

pub fn sum_rollout_proposal_view(view: &RolloutProposalView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_proposal("code-860", "name-359");
        let view = to_rollout_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
