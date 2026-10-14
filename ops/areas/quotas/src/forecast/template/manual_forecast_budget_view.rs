use super::*;

use acme_kit::text;

/// How screens and exports show a manual forecast budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastBudgetView {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub summary: String,
    pub ends_at: String,
    pub issuer: String,
    pub namespace: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_manual_forecast_budget_view(item: &ManualForecastBudget) -> ManualForecastBudgetView {
    ManualForecastBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        fingerprint: item.fingerprint.clone(),
        summary: text::truncate(&item.summary, 80),
        ends_at: item.ends_at.to_string(),
        issuer: item.issuer.clone(),
        namespace: text::truncate(&item.namespace, 64),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_forecast_budget_views(items: &[ManualForecastBudget]) -> Vec<ManualForecastBudgetView> {
    let mut views: Vec<ManualForecastBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_forecast_budget_view(item));
    }
    views
}

pub fn sum_manual_forecast_budget_view(view: &ManualForecastBudgetView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_forecast_budget("code-883", "label-146");
        let view = to_manual_forecast_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_forecast_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
