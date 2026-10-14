use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled health check rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckRuleView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub segment: String,
    pub region: String,
    pub key_id_text: String,
    pub host: String,
    pub ratio: String,
    pub version: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_scheduled_health_check_rule_view(item: &ScheduledHealthCheckRule) -> ScheduledHealthCheckRuleView {
    ScheduledHealthCheckRuleView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        summary: text::truncate(&item.summary, 80),
        segment: item.segment.clone(),
        region: item.region.clone(),
        key_id_text: text::truncate(&item.key_id, 40),
        host: item.host.clone(),
        ratio: format!("{:.2}", item.ratio),
        version: item.version.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_health_check_rule_views(items: &[ScheduledHealthCheckRule]) -> Vec<ScheduledHealthCheckRuleView> {
    let mut views: Vec<ScheduledHealthCheckRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_health_check_rule_view(item));
    }
    views
}

pub fn sum_scheduled_health_check_rule_view(view: &ScheduledHealthCheckRuleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ratio + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_health_check_rule("code-985", "title-321");
        let view = to_scheduled_health_check_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_health_check_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
