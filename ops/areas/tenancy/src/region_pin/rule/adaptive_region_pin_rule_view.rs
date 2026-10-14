use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive region pin rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleView {
    pub code_text: String,
    pub name: String,
    pub owner_id: String,
    pub cost: String,
    pub scope: String,
    pub country: String,
    pub tenant_id_text: String,
    pub revision: String,
    pub timezone: String,
    pub floor_value_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_adaptive_region_pin_rule_view(item: &AdaptiveRegionPinRule) -> AdaptiveRegionPinRuleView {
    AdaptiveRegionPinRuleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        cost: format!("{:.2}", item.cost),
        scope: item.scope.clone(),
        country: item.country.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 64),
        revision: item.revision.to_string(),
        timezone: item.timezone.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_region_pin_rule_views(items: &[AdaptiveRegionPinRule]) -> Vec<AdaptiveRegionPinRuleView> {
    let mut views: Vec<AdaptiveRegionPinRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_region_pin_rule_view(item));
    }
    views
}

pub fn sum_adaptive_region_pin_rule_view(view: &AdaptiveRegionPinRuleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_cost + (parsed_revision as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_region_pin_rule("code-277", "name-221");
        let view = to_adaptive_region_pin_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_region_pin_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
