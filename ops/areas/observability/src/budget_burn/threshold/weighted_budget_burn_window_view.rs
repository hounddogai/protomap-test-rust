use super::*;

use acme_kit::text;

/// How screens and exports show a weighted budget burn window.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnWindowView {
    pub code: String,
    pub title: String,
    pub grace_days: String,
    pub cluster: String,
    pub country_text: String,
    pub starts_at: String,
    pub ratio: String,
    pub limit_text: String,
    pub expires_at: String,
    pub unit: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_weighted_budget_burn_window_view(item: &WeightedBudgetBurnWindow) -> WeightedBudgetBurnWindowView {
    WeightedBudgetBurnWindowView {
        code: item.code.clone(),
        title: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        cluster: text::truncate(&item.cluster, 80),
        country_text: text::truncate(&item.country, 120),
        starts_at: item.starts_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        limit_text: item.limit.to_string(),
        expires_at: item.expires_at.to_string(),
        unit: item.unit.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_budget_burn_window_views(items: &[WeightedBudgetBurnWindow]) -> Vec<WeightedBudgetBurnWindowView> {
    let mut views: Vec<WeightedBudgetBurnWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_budget_burn_window_view(item));
    }
    views
}

pub fn sum_weighted_budget_burn_window_view(view: &WeightedBudgetBurnWindowView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_starts_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_budget_burn_window("code-678", "title-628");
        let view = to_weighted_budget_burn_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_budget_burn_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
