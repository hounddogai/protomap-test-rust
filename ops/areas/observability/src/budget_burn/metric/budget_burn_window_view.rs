use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn window.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnWindowView {
    pub code_text: String,
    pub label: String,
    pub retries: String,
    pub title: String,
    pub ttl_seconds: String,
    pub principal_text: String,
    pub timezone: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_budget_burn_window_view(item: &BudgetBurnWindow) -> BudgetBurnWindowView {
    BudgetBurnWindowView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        principal_text: item.principal.clone(),
        timezone: text::truncate(&item.timezone, 64),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_window_views(items: &[BudgetBurnWindow]) -> Vec<BudgetBurnWindowView> {
    let mut views: Vec<BudgetBurnWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_window_view(item));
    }
    views
}

pub fn sum_budget_burn_window_view(view: &BudgetBurnWindowView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_ttl_seconds as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_window("code-622", "label-678");
        let view = to_budget_burn_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
