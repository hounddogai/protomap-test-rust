use super::*;

/// How screens and exports show a default usage rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageRuleView {
    pub code: String,
    pub label_text: String,
    pub window_days_text: String,
    pub discount: String,
    pub external_ref: String,
    pub starts_at: String,
    pub cost: String,
    pub capacity: String,
    pub key_id: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_default_usage_rule_view(item: &DefaultUsageRule) -> DefaultUsageRuleView {
    DefaultUsageRuleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        window_days_text: item.window_days.to_string(),
        discount: format!("{:.2}", item.discount),
        external_ref: item.external_ref.clone(),
        starts_at: item.starts_at.to_string(),
        cost: format!("{:.2}", item.cost),
        capacity: item.capacity.to_string(),
        key_id: item.key_id.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_usage_rule_views(items: &[DefaultUsageRule]) -> Vec<DefaultUsageRuleView> {
    let mut views: Vec<DefaultUsageRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_usage_rule_view(item));
    }
    views
}

pub fn sum_default_usage_rule_view(view: &DefaultUsageRuleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_discount + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_usage_rule("code-616", "label-368");
        let view = to_default_usage_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_usage_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
