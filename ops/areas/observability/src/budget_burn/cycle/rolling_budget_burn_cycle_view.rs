use super::*;

use acme_kit::text;

/// How screens and exports show a rolling budget burn cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnCycleView {
    pub code_text: String,
    pub name: String,
    pub quota_units_text: String,
    pub limit: String,
    pub ttl_seconds: String,
    pub cipher: String,
    pub ceiling: String,
    pub locale: String,
    pub version: String,
    pub retries_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_rolling_budget_burn_cycle_view(item: &RollingBudgetBurnCycle) -> RollingBudgetBurnCycleView {
    RollingBudgetBurnCycleView {
        code_text: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 64),
        quota_units_text: item.quota_units.to_string(),
        limit: item.limit.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        cipher: text::truncate(&item.cipher, 64),
        ceiling: format!("{:.2}", item.ceiling),
        locale: item.locale.clone(),
        version: item.version.to_string(),
        retries_text: item.retries.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_budget_burn_cycle_views(items: &[RollingBudgetBurnCycle]) -> Vec<RollingBudgetBurnCycleView> {
    let mut views: Vec<RollingBudgetBurnCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_budget_burn_cycle_view(item));
    }
    views
}

pub fn sum_rolling_budget_burn_cycle_view(view: &RollingBudgetBurnCycleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_limit as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_budget_burn_cycle("code-50", "name-167");
        let view = to_rolling_budget_burn_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_budget_burn_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
