use super::*;

/// How screens and exports show a rolling budget burn proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnProposalView {
    pub code: String,
    pub label: String,
    pub ratio_text: String,
    pub segment: String,
    pub source: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_rolling_budget_burn_proposal_view(item: &RollingBudgetBurnProposal) -> RollingBudgetBurnProposalView {
    RollingBudgetBurnProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        segment: item.segment.clone(),
        source: item.source.clone(),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_budget_burn_proposal_views(
    items: &[RollingBudgetBurnProposal],
) -> Vec<RollingBudgetBurnProposalView> {
    let mut views: Vec<RollingBudgetBurnProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_budget_burn_proposal_view(item));
    }
    views
}

pub fn sum_rolling_budget_burn_proposal_view(view: &RollingBudgetBurnProposalView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ratio + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_budget_burn_proposal("code-925", "label-267");
        let view = to_rolling_budget_burn_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_budget_burn_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
