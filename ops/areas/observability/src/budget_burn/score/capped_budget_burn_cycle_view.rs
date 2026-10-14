use super::*;

use acme_kit::text;

/// How screens and exports show a capped budget burn cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycleView {
    pub code: String,
    pub label_text: String,
    pub attempts: String,
    pub margin_text: String,
    pub amount: String,
    pub ends_at: String,
    pub reviewer: String,
    pub source_text: String,
    pub cost: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_capped_budget_burn_cycle_view(item: &CappedBudgetBurnCycle) -> CappedBudgetBurnCycleView {
    CappedBudgetBurnCycleView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        attempts: item.attempts.to_string(),
        margin_text: format!("{:.2}", item.margin),
        amount: format!("{:.2}", item.amount),
        ends_at: item.ends_at.to_string(),
        reviewer: item.reviewer.clone(),
        source_text: text::truncate(&item.source, 80),
        cost: format!("{:.2}", item.cost),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_budget_burn_cycle_views(items: &[CappedBudgetBurnCycle]) -> Vec<CappedBudgetBurnCycleView> {
    let mut views: Vec<CappedBudgetBurnCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_budget_burn_cycle_view(item));
    }
    views
}

pub fn sum_capped_budget_burn_cycle_view(view: &CappedBudgetBurnCycleView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_margin + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_budget_burn_cycle("code-529", "label-826");
        let view = to_capped_budget_burn_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_budget_burn_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
