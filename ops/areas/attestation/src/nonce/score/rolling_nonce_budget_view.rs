use super::*;

use acme_kit::text;

/// How screens and exports show a rolling nonce budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBudgetView {
    pub code_text: String,
    pub name: String,
    pub approved_text: String,
    pub country: String,
    pub retries_text: String,
    pub maximum_text: String,
    pub duration_minutes_text: String,
    pub version: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_rolling_nonce_budget_view(item: &RollingNonceBudget) -> RollingNonceBudgetView {
    RollingNonceBudgetView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        approved_text: format!("{}", item.approved),
        country: item.country.clone(),
        retries_text: item.retries.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        duration_minutes_text: item.duration_minutes.to_string(),
        version: item.version.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_nonce_budget_views(items: &[RollingNonceBudget]) -> Vec<RollingNonceBudgetView> {
    let mut views: Vec<RollingNonceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_nonce_budget_view(item));
    }
    views
}

pub fn sum_rolling_nonce_budget_view(view: &RollingNonceBudgetView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_retries as f64 + parsed_maximum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_nonce_budget("code-400", "name-751");
        let view = to_rolling_nonce_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_nonce_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
