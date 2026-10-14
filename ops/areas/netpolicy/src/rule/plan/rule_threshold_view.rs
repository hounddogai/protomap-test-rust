use super::*;

use acme_kit::text;

/// How screens and exports show a rule threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleThresholdView {
    pub code_text: String,
    pub label_text: String,
    pub source: String,
    pub cost: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_rule_threshold_view(item: &RuleThreshold) -> RuleThresholdView {
    RuleThresholdView {
        code_text: text::truncate(&item.code, 64),
        label_text: text::truncate(&item.label, 120),
        source: text::truncate(&item.source, 120),
        cost: format!("{:.2}", item.cost),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_threshold_views(items: &[RuleThreshold]) -> Vec<RuleThresholdView> {
    let mut views: Vec<RuleThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_threshold_view(item));
    }
    views
}

pub fn sum_rule_threshold_view(view: &RuleThresholdView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_cost + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_threshold("code-286", "label-609");
        let view = to_rule_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
