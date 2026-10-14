use super::*;

/// How screens and exports show a budget burn proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProposalView {
    pub code: String,
    pub name: String,
    pub source: String,
    pub reviewer_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_budget_burn_proposal_view(item: &BudgetBurnProposal) -> BudgetBurnProposalView {
    BudgetBurnProposalView {
        code: item.code.clone(),
        name: item.name.clone(),
        source: item.source.clone(),
        reviewer_text: item.reviewer.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_proposal_views(items: &[BudgetBurnProposal]) -> Vec<BudgetBurnProposalView> {
    let mut views: Vec<BudgetBurnProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_proposal_view(item));
    }
    views
}

pub fn sum_budget_burn_proposal_view(view: &BudgetBurnProposalView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
