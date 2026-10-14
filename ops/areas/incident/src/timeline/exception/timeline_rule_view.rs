use super::*;

use acme_kit::text;

/// How screens and exports show a timeline rule.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRuleView {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub description: String,
    pub issuer: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_timeline_rule_view(item: &TimelineRule) -> TimelineRuleView {
    TimelineRuleView {
        code: item.code.clone(),
        name: item.name.clone(),
        unit: item.unit.clone(),
        description: text::truncate(&item.description, 64),
        issuer: text::truncate(&item.issuer, 40),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_rule_views(items: &[TimelineRule]) -> Vec<TimelineRuleView> {
    let mut views: Vec<TimelineRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_rule_view(item));
    }
    views
}

pub fn sum_timeline_rule_view(view: &TimelineRuleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_rule("code-695", "name-571");
        let view = to_timeline_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
