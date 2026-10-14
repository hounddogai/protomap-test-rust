use super::*;

/// How screens and exports show a adaptive budget burn threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBudgetBurnThresholdView {
    pub code_text: String,
    pub label_text: String,
    pub archived: String,
    pub pinned: String,
    pub country: String,
    pub visible: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_adaptive_budget_burn_threshold_view(item: &AdaptiveBudgetBurnThreshold) -> AdaptiveBudgetBurnThresholdView {
    AdaptiveBudgetBurnThresholdView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        archived: format!("{}", item.archived),
        pinned: format!("{}", item.pinned),
        country: item.country.clone(),
        visible: format!("{}", item.visible),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_budget_burn_threshold_views(
    items: &[AdaptiveBudgetBurnThreshold],
) -> Vec<AdaptiveBudgetBurnThresholdView> {
    let mut views: Vec<AdaptiveBudgetBurnThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_budget_burn_threshold_view(item));
    }
    views
}

pub fn sum_adaptive_budget_burn_threshold_view(view: &AdaptiveBudgetBurnThresholdView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_budget_burn_threshold("code-273", "label-157");
        let view = to_adaptive_budget_burn_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_budget_burn_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
