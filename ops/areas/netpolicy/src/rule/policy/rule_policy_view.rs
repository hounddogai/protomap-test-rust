use super::*;

/// How screens and exports show a rule policy.
#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicyView {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub notes: String,
    pub version: String,
    pub window_days: String,
    pub description_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_rule_policy_view(item: &RulePolicy) -> RulePolicyView {
    RulePolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        audience: item.audience.clone(),
        notes: item.notes.clone(),
        version: item.version.to_string(),
        window_days: item.window_days.to_string(),
        description_text: item.description.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_policy_views(items: &[RulePolicy]) -> Vec<RulePolicyView> {
    let mut views: Vec<RulePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_policy_view(item));
    }
    views
}

pub fn sum_rule_policy_view(view: &RulePolicyView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_version as f64 + (parsed_window_days as f64) + parsed_rate)
}
