use super::*;

use acme_kit::text;

/// How screens and exports show a region pin rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRuleView {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub floor_value: String,
    pub region: String,
    pub limit: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_region_pin_rule_view(item: &RegionPinRule) -> RegionPinRuleView {
    RegionPinRuleView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 64),
        scope: text::truncate(&item.scope, 64),
        floor_value: format!("{:.2}", item.floor_value),
        region: item.region.clone(),
        limit: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_rule_views(items: &[RegionPinRule]) -> Vec<RegionPinRuleView> {
    let mut views: Vec<RegionPinRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_rule_view(item));
    }
    views
}

pub fn sum_region_pin_rule_view(view: &RegionPinRuleView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_floor_value + (parsed_limit as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_rule("code-582", "name-723");
        let view = to_region_pin_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
