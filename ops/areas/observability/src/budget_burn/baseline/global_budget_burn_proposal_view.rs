use super::*;

use acme_kit::text;

/// How screens and exports show a global budget burn proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnProposalView {
    pub code: String,
    pub name_text: String,
    pub cluster: String,
    pub namespace: String,
    pub budget: String,
    pub archived: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_global_budget_burn_proposal_view(item: &GlobalBudgetBurnProposal) -> GlobalBudgetBurnProposalView {
    GlobalBudgetBurnProposalView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        cluster: item.cluster.clone(),
        namespace: item.namespace.clone(),
        budget: format!("{:.2}", item.budget),
        archived: format!("{}", item.archived),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_budget_burn_proposal_views(items: &[GlobalBudgetBurnProposal]) -> Vec<GlobalBudgetBurnProposalView> {
    let mut views: Vec<GlobalBudgetBurnProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_budget_burn_proposal_view(item));
    }
    views
}

pub fn sum_global_budget_burn_proposal_view(view: &GlobalBudgetBurnProposalView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_budget_burn_proposal("code-71", "name-804");
        let view = to_global_budget_burn_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_budget_burn_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
