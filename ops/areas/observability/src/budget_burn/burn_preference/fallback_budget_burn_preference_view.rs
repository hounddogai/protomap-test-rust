use super::*;

use acme_kit::text;

/// How screens and exports show a fallback budget burn preference.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnPreferenceView {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub expires_at_text: String,
    pub summary: String,
    pub score_text: String,
    pub rate: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_fallback_budget_burn_preference_view(
    item: &FallbackBudgetBurnPreference,
) -> FallbackBudgetBurnPreferenceView {
    FallbackBudgetBurnPreferenceView {
        code: item.code.clone(),
        name: item.name.clone(),
        scope: item.scope.clone(),
        expires_at_text: item.expires_at.to_string(),
        summary: text::truncate(&item.summary, 120),
        score_text: format!("{:.2}", item.score),
        rate: format!("{:.2}", item.rate),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_budget_burn_preference_views(
    items: &[FallbackBudgetBurnPreference],
) -> Vec<FallbackBudgetBurnPreferenceView> {
    let mut views: Vec<FallbackBudgetBurnPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_budget_burn_preference_view(item));
    }
    views
}

pub fn sum_fallback_budget_burn_preference_view(view: &FallbackBudgetBurnPreferenceView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_score + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_budget_burn_preference("code-529", "name-629");
        let view = to_fallback_budget_burn_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_budget_burn_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
