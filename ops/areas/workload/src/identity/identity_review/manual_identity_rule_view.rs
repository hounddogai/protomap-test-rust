use super::*;

use acme_kit::text;

/// How screens and exports show a manual identity rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityRuleView {
    pub code: String,
    pub name: String,
    pub cost_text: String,
    pub quantity: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_manual_identity_rule_view(item: &ManualIdentityRule) -> ManualIdentityRuleView {
    ManualIdentityRuleView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        cost_text: format!("{:.2}", item.cost),
        quantity: item.quantity.to_string(),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_identity_rule_views(items: &[ManualIdentityRule]) -> Vec<ManualIdentityRuleView> {
    let mut views: Vec<ManualIdentityRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_identity_rule_view(item));
    }
    views
}

pub fn sum_manual_identity_rule_view(view: &ManualIdentityRuleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_cost + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_identity_rule("code-63", "name-175");
        let view = to_manual_identity_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_identity_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
