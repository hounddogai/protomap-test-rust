use super::*;

use acme_kit::text;

/// How screens and exports show a grant rule.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRuleView {
    pub code_text: String,
    pub title_text: String,
    pub channel: String,
    pub window_days: String,
    pub weight_text: String,
    pub grace_days: String,
    pub owner_id: String,
    pub tenant_id_text: String,
    pub archived: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_grant_rule_view(item: &GrantRule) -> GrantRuleView {
    GrantRuleView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        channel: item.channel.clone(),
        window_days: item.window_days.to_string(),
        weight_text: format!("{:.2}", item.weight),
        grace_days: item.grace_days.to_string(),
        owner_id: item.owner_id.clone(),
        tenant_id_text: item.tenant_id.clone(),
        archived: format!("{}", item.archived),
        segment: text::truncate(&item.segment, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_rule_views(items: &[GrantRule]) -> Vec<GrantRuleView> {
    let mut views: Vec<GrantRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_rule_view(item));
    }
    views
}

pub fn sum_grant_rule_view(view: &GrantRuleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_weight + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_rule("code-922", "title-497");
        let view = to_grant_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
