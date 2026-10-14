use super::*;

use acme_kit::text;

/// How screens and exports show a rule estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleEstimateView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub rotation_days_text: String,
    pub contact_email_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_rule_estimate_view(item: &RuleEstimate) -> RuleEstimateView {
    RuleEstimateView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        batch_size: item.batch_size.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        contact_email_text: item.contact_email.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_estimate_views(items: &[RuleEstimate]) -> Vec<RuleEstimateView> {
    let mut views: Vec<RuleEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_estimate_view(item));
    }
    views
}

pub fn sum_rule_estimate_view(view: &RuleEstimateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_estimate("code-79", "label-261");
        let view = to_rule_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
