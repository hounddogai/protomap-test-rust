use super::*;

use acme_kit::text;

/// How screens and exports show a batched rule adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAdjustmentView {
    pub code_text: String,
    pub title: String,
    pub weight: String,
    pub priority: String,
    pub segment: String,
    pub window_days_text: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_batched_rule_adjustment_view(item: &BatchedRuleAdjustment) -> BatchedRuleAdjustmentView {
    BatchedRuleAdjustmentView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        priority: item.priority.to_string(),
        segment: text::truncate(&item.segment, 120),
        window_days_text: item.window_days.to_string(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_rule_adjustment_views(items: &[BatchedRuleAdjustment]) -> Vec<BatchedRuleAdjustmentView> {
    let mut views: Vec<BatchedRuleAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_rule_adjustment_view(item));
    }
    views
}

pub fn sum_batched_rule_adjustment_view(view: &BatchedRuleAdjustmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_rule_adjustment("code-414", "title-886");
        let view = to_batched_rule_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_rule_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
