use super::*;

/// How screens and exports show a rule rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRollupView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub key_id: String,
    pub duration_minutes_text: String,
    pub priority_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_rule_rollup_view(item: &RuleRollup) -> RuleRollupView {
    RuleRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        cluster: item.cluster.clone(),
        key_id: item.key_id.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        priority_text: item.priority.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_rollup_views(items: &[RuleRollup]) -> Vec<RuleRollupView> {
    let mut views: Vec<RuleRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_rollup_view(item));
    }
    views
}

pub fn sum_rule_rollup_view(view: &RuleRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_rollup("code-213", "name-916");
        let view = to_rule_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
