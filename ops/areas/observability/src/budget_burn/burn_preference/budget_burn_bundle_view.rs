use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBundleView {
    pub code: String,
    pub name: String,
    pub rotation_days: String,
    pub score: String,
    pub floor_value: String,
    pub host: String,
    pub label: String,
    pub rate: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_budget_burn_bundle_view(item: &BudgetBurnBundle) -> BudgetBurnBundleView {
    BudgetBurnBundleView {
        code: item.code.clone(),
        name: item.name.clone(),
        rotation_days: item.rotation_days.to_string(),
        score: format!("{:.2}", item.score),
        floor_value: format!("{:.2}", item.floor_value),
        host: item.host.clone(),
        label: text::truncate(&item.label, 80),
        rate: format!("{:.2}", item.rate),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_bundle_views(items: &[BudgetBurnBundle]) -> Vec<BudgetBurnBundleView> {
    let mut views: Vec<BudgetBurnBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_bundle_view(item));
    }
    views
}

pub fn sum_budget_burn_bundle_view(view: &BudgetBurnBundleView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_score + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_bundle("code-293", "name-659");
        let view = to_budget_burn_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
