use super::*;

/// How screens and exports show a manual budget burn proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnProposalView {
    pub code_text: String,
    pub title: String,
    pub window_days_text: String,
    pub minimum: String,
    pub threshold_text: String,
    pub category: String,
    pub visible: String,
    pub active_text: String,
    pub segment_text: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_manual_budget_burn_proposal_view(item: &ManualBudgetBurnProposal) -> ManualBudgetBurnProposalView {
    ManualBudgetBurnProposalView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        window_days_text: item.window_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        threshold_text: format!("{:.2}", item.threshold),
        category: item.category.clone(),
        visible: format!("{}", item.visible),
        active_text: format!("{}", item.active),
        segment_text: item.segment.clone(),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_budget_burn_proposal_views(items: &[ManualBudgetBurnProposal]) -> Vec<ManualBudgetBurnProposalView> {
    let mut views: Vec<ManualBudgetBurnProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_budget_burn_proposal_view(item));
    }
    views
}

pub fn sum_manual_budget_burn_proposal_view(view: &ManualBudgetBurnProposalView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_minimum + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_budget_burn_proposal("code-658", "title-824");
        let view = to_manual_budget_burn_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_budget_burn_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
