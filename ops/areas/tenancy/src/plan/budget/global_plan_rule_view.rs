use super::*;

use acme_kit::text;

/// How screens and exports show a global plan rule.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanRuleView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub locale: String,
    pub margin: String,
    pub sequence: String,
    pub rate: String,
    pub maximum: String,
    pub category: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_global_plan_rule_view(item: &GlobalPlanRule) -> GlobalPlanRuleView {
    GlobalPlanRuleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        locale: item.locale.clone(),
        margin: format!("{:.2}", item.margin),
        sequence: item.sequence.to_string(),
        rate: format!("{:.2}", item.rate),
        maximum: format!("{:.2}", item.maximum),
        category: text::truncate(&item.category, 80),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_plan_rule_views(items: &[GlobalPlanRule]) -> Vec<GlobalPlanRuleView> {
    let mut views: Vec<GlobalPlanRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_plan_rule_view(item));
    }
    views
}

pub fn sum_global_plan_rule_view(view: &GlobalPlanRuleView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_margin + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_plan_rule("code-278", "name-372");
        let view = to_global_plan_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_plan_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
