use super::*;

/// How screens and exports show a lenient flow log proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogProposalView {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_lenient_flow_log_proposal_view(item: &LenientFlowLogProposal) -> LenientFlowLogProposalView {
    LenientFlowLogProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        unit: item.unit.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_flow_log_proposal_views(items: &[LenientFlowLogProposal]) -> Vec<LenientFlowLogProposalView> {
    let mut views: Vec<LenientFlowLogProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_flow_log_proposal_view(item));
    }
    views
}

pub fn sum_lenient_flow_log_proposal_view(view: &LenientFlowLogProposalView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_flow_log_proposal("code-426", "label-967");
        let view = to_lenient_flow_log_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_flow_log_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
