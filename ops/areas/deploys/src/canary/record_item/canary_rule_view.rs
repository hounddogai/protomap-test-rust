use super::*;

use acme_kit::text;

/// How screens and exports show a canary rule.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRuleView {
    pub code_text: String,
    pub name_text: String,
    pub principal: String,
    pub timezone: String,
    pub pinned: String,
    pub audience_text: String,
    pub priority_text: String,
    pub external_ref: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_canary_rule_view(item: &CanaryRule) -> CanaryRuleView {
    CanaryRuleView {
        code_text: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 40),
        principal: text::truncate(&item.principal, 80),
        timezone: item.timezone.clone(),
        pinned: format!("{}", item.pinned),
        audience_text: text::truncate(&item.audience, 40),
        priority_text: item.priority.to_string(),
        external_ref: item.external_ref.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_rule_views(items: &[CanaryRule]) -> Vec<CanaryRuleView> {
    let mut views: Vec<CanaryRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_rule_view(item));
    }
    views
}

pub fn sum_canary_rule_view(view: &CanaryRuleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_priority as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_rule("code-993", "name-553");
        let view = to_canary_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
