use super::*;

/// How screens and exports show a primary egress proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposalView {
    pub code_text: String,
    pub name: String,
    pub margin_text: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_primary_egress_proposal_view(item: &PrimaryEgressProposal) -> PrimaryEgressProposalView {
    PrimaryEgressProposalView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_egress_proposal_views(items: &[PrimaryEgressProposal]) -> Vec<PrimaryEgressProposalView> {
    let mut views: Vec<PrimaryEgressProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_egress_proposal_view(item));
    }
    views
}

pub fn sum_primary_egress_proposal_view(view: &PrimaryEgressProposalView) -> Result<f64, String> {
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
        let item = new_primary_egress_proposal("code-182", "name-983");
        let view = to_primary_egress_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_egress_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
