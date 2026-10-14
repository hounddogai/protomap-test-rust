use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyThresholdView {
    pub code: String,
    pub label: String,
    pub budget_text: String,
    pub maximum: String,
    pub country: String,
    pub discount: String,
    pub quota_units: String,
    pub priority_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_ttl_policy_threshold_view(item: &TtlPolicyThreshold) -> TtlPolicyThresholdView {
    TtlPolicyThresholdView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        budget_text: format!("{:.2}", item.budget),
        maximum: format!("{:.2}", item.maximum),
        country: text::truncate(&item.country, 120),
        discount: format!("{:.2}", item.discount),
        quota_units: item.quota_units.to_string(),
        priority_text: item.priority.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_threshold_views(items: &[TtlPolicyThreshold]) -> Vec<TtlPolicyThresholdView> {
    let mut views: Vec<TtlPolicyThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_threshold_view(item));
    }
    views
}

pub fn sum_ttl_policy_threshold_view(view: &TtlPolicyThresholdView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_budget + parsed_maximum + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_threshold("code-673", "label-470");
        let view = to_ttl_policy_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
