use super::*;

use acme_kit::text;

/// How screens and exports show a runbook rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookRuleView {
    pub code: String,
    pub title_text: String,
    pub burst: String,
    pub host: String,
    pub priority: String,
    pub attempts_text: String,
    pub cost_text: String,
    pub retries: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_runbook_rule_view(item: &RunbookRule) -> RunbookRuleView {
    RunbookRuleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        burst: item.burst.to_string(),
        host: item.host.clone(),
        priority: item.priority.to_string(),
        attempts_text: item.attempts.to_string(),
        cost_text: format!("{:.2}", item.cost),
        retries: item.retries.to_string(),
        principal_text: text::truncate(&item.principal, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_rule_views(items: &[RunbookRule]) -> Vec<RunbookRuleView> {
    let mut views: Vec<RunbookRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_rule_view(item));
    }
    views
}

pub fn sum_runbook_rule_view(view: &RunbookRuleView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_priority as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_rule("code-268", "title-139");
        let view = to_runbook_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
