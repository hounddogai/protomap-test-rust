use super::*;

use acme_kit::text;

/// How screens and exports show a regional suspension budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBudgetView {
    pub code: String,
    pub name: String,
    pub reviewer_text: String,
    pub quota_units: String,
    pub score: String,
    pub port: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_regional_suspension_budget_view(item: &RegionalSuspensionBudget) -> RegionalSuspensionBudgetView {
    RegionalSuspensionBudgetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        reviewer_text: item.reviewer.clone(),
        quota_units: item.quota_units.to_string(),
        score: format!("{:.2}", item.score),
        port: item.port.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_suspension_budget_views(items: &[RegionalSuspensionBudget]) -> Vec<RegionalSuspensionBudgetView> {
    let mut views: Vec<RegionalSuspensionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_suspension_budget_view(item));
    }
    views
}

pub fn sum_regional_suspension_budget_view(view: &RegionalSuspensionBudgetView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_score + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_suspension_budget("code-192", "name-357");
        let view = to_regional_suspension_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_suspension_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
