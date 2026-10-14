use super::*;

use acme_kit::text;

/// How screens and exports show a plan rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleView {
    pub code_text: String,
    pub title: String,
    pub burst: String,
    pub key_id_text: String,
    pub ceiling_text: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_plan_rule_view(item: &PlanRule) -> PlanRuleView {
    PlanRuleView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        burst: item.burst.to_string(),
        key_id_text: text::truncate(&item.key_id, 40),
        ceiling_text: format!("{:.2}", item.ceiling),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_rule_views(items: &[PlanRule]) -> Vec<PlanRuleView> {
    let mut views: Vec<PlanRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_rule_view(item));
    }
    views
}

pub fn sum_plan_rule_view(view: &PlanRuleView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_burst as f64 + parsed_ceiling + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_rule("code-555", "title-434");
        let view = to_plan_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
