use super::*;

/// How screens and exports show a action item proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemProposalView {
    pub code_text: String,
    pub name: String,
    pub revision: String,
    pub sequence_text: String,
    pub category: String,
    pub contact_email: String,
    pub port_text: String,
    pub limit_text: String,
    pub rate: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_action_item_proposal_view(item: &ActionItemProposal) -> ActionItemProposalView {
    ActionItemProposalView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        sequence_text: item.sequence.to_string(),
        category: item.category.clone(),
        contact_email: item.contact_email.clone(),
        port_text: item.port.to_string(),
        limit_text: item.limit.to_string(),
        rate: format!("{:.2}", item.rate),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_proposal_views(items: &[ActionItemProposal]) -> Vec<ActionItemProposalView> {
    let mut views: Vec<ActionItemProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_proposal_view(item));
    }
    views
}

pub fn sum_action_item_proposal_view(view: &ActionItemProposalView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_sequence as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_proposal("code-988", "name-504");
        let view = to_action_item_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
