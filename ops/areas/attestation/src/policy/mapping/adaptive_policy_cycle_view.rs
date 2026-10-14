use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive policy cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicyCycleView {
    pub code: String,
    pub label_text: String,
    pub category: String,
    pub margin: String,
    pub discount: String,
    pub attempts: String,
    pub channel: String,
    pub tenant_id_text: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_adaptive_policy_cycle_view(item: &AdaptivePolicyCycle) -> AdaptivePolicyCycleView {
    AdaptivePolicyCycleView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        category: item.category.clone(),
        margin: format!("{:.2}", item.margin),
        discount: format!("{:.2}", item.discount),
        attempts: item.attempts.to_string(),
        channel: text::truncate(&item.channel, 80),
        tenant_id_text: item.tenant_id.clone(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_policy_cycle_views(items: &[AdaptivePolicyCycle]) -> Vec<AdaptivePolicyCycleView> {
    let mut views: Vec<AdaptivePolicyCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_policy_cycle_view(item));
    }
    views
}

pub fn sum_adaptive_policy_cycle_view(view: &AdaptivePolicyCycleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_margin + parsed_discount + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_policy_cycle("code-131", "label-31");
        let view = to_adaptive_policy_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_policy_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
