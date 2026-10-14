use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal budget burn estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateView {
    pub code_text: String,
    pub title: String,
    pub capacity: String,
    pub key_id: String,
    pub phone: String,
    pub threshold_text: String,
    pub score: String,
    pub ratio: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_seasonal_budget_burn_estimate_view(item: &SeasonalBudgetBurnEstimate) -> SeasonalBudgetBurnEstimateView {
    SeasonalBudgetBurnEstimateView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        capacity: item.capacity.to_string(),
        key_id: item.key_id.clone(),
        phone: text::truncate(&item.phone, 40),
        threshold_text: format!("{:.2}", item.threshold),
        score: format!("{:.2}", item.score),
        ratio: format!("{:.2}", item.ratio),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_budget_burn_estimate_views(
    items: &[SeasonalBudgetBurnEstimate],
) -> Vec<SeasonalBudgetBurnEstimateView> {
    let mut views: Vec<SeasonalBudgetBurnEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_budget_burn_estimate_view(item));
    }
    views
}

pub fn sum_seasonal_budget_burn_estimate_view(view: &SeasonalBudgetBurnEstimateView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_threshold + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_budget_burn_estimate("code-268", "title-417");
        let view = to_seasonal_budget_burn_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_budget_burn_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
