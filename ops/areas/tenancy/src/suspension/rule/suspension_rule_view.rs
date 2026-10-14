use super::*;

use acme_kit::text;

/// How screens and exports show a suspension rule.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRuleView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub grace_days: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_suspension_rule_view(item: &SuspensionRule) -> SuspensionRuleView {
    SuspensionRuleView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_rule_views(items: &[SuspensionRule]) -> Vec<SuspensionRuleView> {
    let mut views: Vec<SuspensionRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_rule_view(item));
    }
    views
}

pub fn sum_suspension_rule_view(view: &SuspensionRuleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_rule("code-704", "name-225");
        let view = to_suspension_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
