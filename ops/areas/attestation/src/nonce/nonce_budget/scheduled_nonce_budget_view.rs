use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled nonce budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceBudgetView {
    pub code: String,
    pub name_text: String,
    pub burst: String,
    pub issuer: String,
    pub threshold: String,
    pub currency_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_scheduled_nonce_budget_view(item: &ScheduledNonceBudget) -> ScheduledNonceBudgetView {
    ScheduledNonceBudgetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        burst: item.burst.to_string(),
        issuer: text::truncate(&item.issuer, 120),
        threshold: format!("{:.2}", item.threshold),
        currency_text: item.currency.clone(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_nonce_budget_views(items: &[ScheduledNonceBudget]) -> Vec<ScheduledNonceBudgetView> {
    let mut views: Vec<ScheduledNonceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_nonce_budget_view(item));
    }
    views
}

pub fn sum_scheduled_nonce_budget_view(view: &ScheduledNonceBudgetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_burst as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_nonce_budget("code-780", "name-3");
        let view = to_scheduled_nonce_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_nonce_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
