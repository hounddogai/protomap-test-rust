use super::*;

/// How screens and exports show a legacy flow log proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogProposalView {
    pub code_text: String,
    pub name_text: String,
    pub attempts: String,
    pub description_text: String,
    pub threshold: String,
    pub quantity: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_legacy_flow_log_proposal_view(item: &LegacyFlowLogProposal) -> LegacyFlowLogProposalView {
    LegacyFlowLogProposalView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        attempts: item.attempts.to_string(),
        description_text: item.description.clone(),
        threshold: format!("{:.2}", item.threshold),
        quantity: item.quantity.to_string(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_flow_log_proposal_views(items: &[LegacyFlowLogProposal]) -> Vec<LegacyFlowLogProposalView> {
    let mut views: Vec<LegacyFlowLogProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_flow_log_proposal_view(item));
    }
    views
}

pub fn sum_legacy_flow_log_proposal_view(view: &LegacyFlowLogProposalView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_threshold + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_flow_log_proposal("code-355", "name-912");
        let view = to_legacy_flow_log_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_flow_log_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
