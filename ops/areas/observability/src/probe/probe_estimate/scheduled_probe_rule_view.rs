use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled probe rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRuleView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub pinned: String,
    pub namespace: String,
    pub budget_text: String,
    pub starts_at: String,
    pub label: String,
    pub discount_text: String,
    pub segment_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_scheduled_probe_rule_view(item: &ScheduledProbeRule) -> ScheduledProbeRuleView {
    ScheduledProbeRuleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        pinned: format!("{}", item.pinned),
        namespace: item.namespace.clone(),
        budget_text: format!("{:.2}", item.budget),
        starts_at: item.starts_at.to_string(),
        label: item.label.clone(),
        discount_text: format!("{:.2}", item.discount),
        segment_text: text::truncate(&item.segment, 120),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_probe_rule_views(items: &[ScheduledProbeRule]) -> Vec<ScheduledProbeRuleView> {
    let mut views: Vec<ScheduledProbeRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_probe_rule_view(item));
    }
    views
}

pub fn sum_scheduled_probe_rule_view(view: &ScheduledProbeRuleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_budget + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_probe_rule("code-363", "name-543");
        let view = to_scheduled_probe_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_probe_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
