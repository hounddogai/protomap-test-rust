use super::*;

use acme_kit::text;

/// How screens and exports show a default timeline rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineRuleView {
    pub code: String,
    pub name: String,
    pub cost_text: String,
    pub port: String,
    pub version_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_default_timeline_rule_view(item: &DefaultTimelineRule) -> DefaultTimelineRuleView {
    DefaultTimelineRuleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        cost_text: format!("{:.2}", item.cost),
        port: item.port.to_string(),
        version_text: item.version.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_timeline_rule_views(items: &[DefaultTimelineRule]) -> Vec<DefaultTimelineRuleView> {
    let mut views: Vec<DefaultTimelineRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_timeline_rule_view(item));
    }
    views
}

pub fn sum_default_timeline_rule_view(view: &DefaultTimelineRuleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_cost + (parsed_port as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_timeline_rule("code-339", "name-184");
        let view = to_default_timeline_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_timeline_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
