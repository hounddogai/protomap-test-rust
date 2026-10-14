use super::*;

use acme_kit::text;

/// How screens and exports show a manual slo budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloBudgetView {
    pub code: String,
    pub name: String,
    pub pinned: String,
    pub reviewer: String,
    pub version: String,
    pub label: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_manual_slo_budget_view(item: &ManualSloBudget) -> ManualSloBudgetView {
    ManualSloBudgetView {
        code: item.code.clone(),
        name: item.name.clone(),
        pinned: format!("{}", item.pinned),
        reviewer: text::truncate(&item.reviewer, 120),
        version: item.version.to_string(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_slo_budget_views(items: &[ManualSloBudget]) -> Vec<ManualSloBudgetView> {
    let mut views: Vec<ManualSloBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_slo_budget_view(item));
    }
    views
}

pub fn sum_manual_slo_budget_view(view: &ManualSloBudgetView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_version as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_slo_budget("code-195", "name-793");
        let view = to_manual_slo_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_slo_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
