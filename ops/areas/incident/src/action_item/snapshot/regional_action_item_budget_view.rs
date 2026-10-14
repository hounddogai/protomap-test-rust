use super::*;

use acme_kit::text;

/// How screens and exports show a regional action item budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemBudgetView {
    pub code: String,
    pub name: String,
    pub port: String,
    pub margin: String,
    pub updated_at: String,
    pub floor_value: String,
    pub reviewer: String,
    pub minimum: String,
    pub tenant_id: String,
    pub region: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_regional_action_item_budget_view(item: &RegionalActionItemBudget) -> RegionalActionItemBudgetView {
    RegionalActionItemBudgetView {
        code: item.code.clone(),
        name: item.name.clone(),
        port: item.port.to_string(),
        margin: format!("{:.2}", item.margin),
        updated_at: item.updated_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        reviewer: text::truncate(&item.reviewer, 40),
        minimum: format!("{:.2}", item.minimum),
        tenant_id: item.tenant_id.clone(),
        region: item.region.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_action_item_budget_views(items: &[RegionalActionItemBudget]) -> Vec<RegionalActionItemBudgetView> {
    let mut views: Vec<RegionalActionItemBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_action_item_budget_view(item));
    }
    views
}

pub fn sum_regional_action_item_budget_view(view: &RegionalActionItemBudgetView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_port as f64 + parsed_margin + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_action_item_budget("code-388", "name-910");
        let view = to_regional_action_item_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_action_item_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
