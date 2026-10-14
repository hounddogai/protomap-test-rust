use super::*;

use acme_kit::text;

/// How screens and exports show a health check budget.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetView {
    pub code: String,
    pub label: String,
    pub margin_text: String,
    pub namespace: String,
    pub version_text: String,
    pub cost: String,
    pub reviewer: String,
    pub ttl_seconds: String,
    pub locale_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_health_check_budget_view(item: &HealthCheckBudget) -> HealthCheckBudgetView {
    HealthCheckBudgetView {
        code: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 120),
        margin_text: format!("{:.2}", item.margin),
        namespace: item.namespace.clone(),
        version_text: item.version.to_string(),
        cost: format!("{:.2}", item.cost),
        reviewer: item.reviewer.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        locale_text: item.locale.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_budget_views(items: &[HealthCheckBudget]) -> Vec<HealthCheckBudgetView> {
    let mut views: Vec<HealthCheckBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_budget_view(item));
    }
    views
}

pub fn sum_health_check_budget_view(view: &HealthCheckBudgetView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_margin + (parsed_version as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_budget("code-470", "label-339");
        let view = to_health_check_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
