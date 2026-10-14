use super::*;

use acme_kit::text;

/// How screens and exports show a rolling allowlist exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub cluster: String,
    pub key_id: String,
    pub region_text: String,
    pub attempts: String,
    pub budget_text: String,
    pub archived: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_rolling_allowlist_exception_view(item: &RollingAllowlistException) -> RollingAllowlistExceptionView {
    RollingAllowlistExceptionView {
        code: item.code.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        cluster: item.cluster.clone(),
        key_id: text::truncate(&item.key_id, 64),
        region_text: text::truncate(&item.region, 80),
        attempts: item.attempts.to_string(),
        budget_text: format!("{:.2}", item.budget),
        archived: format!("{}", item.archived),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_allowlist_exception_views(items: &[RollingAllowlistException]) -> Vec<RollingAllowlistExceptionView> {
    let mut views: Vec<RollingAllowlistExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_allowlist_exception_view(item));
    }
    views
}

pub fn sum_rolling_allowlist_exception_view(view: &RollingAllowlistExceptionView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_attempts as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_allowlist_exception("code-14", "name-237");
        let view = to_rolling_allowlist_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_allowlist_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
