use super::*;

use acme_kit::text;

/// How screens and exports show a slo budget.
#[derive(Debug, Clone, PartialEq)]
pub struct SloBudgetView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub quota_units_text: String,
    pub locale: String,
    pub port_text: String,
    pub starts_at: String,
    pub fingerprint: String,
    pub floor_value: String,
    pub discount_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_slo_budget_view(item: &SloBudget) -> SloBudgetView {
    SloBudgetView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        host: text::truncate(&item.host, 120),
        quota_units_text: item.quota_units.to_string(),
        locale: text::truncate(&item.locale, 64),
        port_text: item.port.to_string(),
        starts_at: item.starts_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        floor_value: format!("{:.2}", item.floor_value),
        discount_text: format!("{:.2}", item.discount),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_budget_views(items: &[SloBudget]) -> Vec<SloBudgetView> {
    let mut views: Vec<SloBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_budget_view(item));
    }
    views
}

pub fn sum_slo_budget_view(view: &SloBudgetView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_port as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_budget("code-578", "title-623");
        let view = to_slo_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
