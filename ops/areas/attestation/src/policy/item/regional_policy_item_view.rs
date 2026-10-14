use super::*;

use acme_kit::text;

/// How screens and exports show a regional policy item.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyItemView {
    pub code_text: String,
    pub name_text: String,
    pub margin: String,
    pub discount: String,
    pub scope: String,
    pub created_at: String,
    pub reviewer: String,
    pub issuer_text: String,
    pub visible_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_regional_policy_item_view(item: &RegionalPolicyItem) -> RegionalPolicyItemView {
    RegionalPolicyItemView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        discount: format!("{:.2}", item.discount),
        scope: text::truncate(&item.scope, 120),
        created_at: item.created_at.to_string(),
        reviewer: text::truncate(&item.reviewer, 64),
        issuer_text: item.issuer.clone(),
        visible_text: format!("{}", item.visible),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_policy_item_views(items: &[RegionalPolicyItem]) -> Vec<RegionalPolicyItemView> {
    let mut views: Vec<RegionalPolicyItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_policy_item_view(item));
    }
    views
}

pub fn sum_regional_policy_item_view(view: &RegionalPolicyItemView) -> Result<f64, String> {
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
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_margin + parsed_discount + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_policy_item("code-218", "name-765");
        let view = to_regional_policy_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_policy_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
