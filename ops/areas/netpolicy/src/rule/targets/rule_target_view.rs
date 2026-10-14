use super::*;

use acme_kit::text;

/// How screens and exports show a rule target.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleTargetView {
    pub code_text: String,
    pub title_text: String,
    pub timezone: String,
    pub attempts: String,
    pub namespace_text: String,
    pub version: String,
    pub currency: String,
    pub expires_at_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_rule_target_view(item: &RuleTarget) -> RuleTargetView {
    RuleTargetView {
        code_text: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        timezone: text::truncate(&item.timezone, 64),
        attempts: item.attempts.to_string(),
        namespace_text: text::truncate(&item.namespace, 40),
        version: item.version.to_string(),
        currency: item.currency.clone(),
        expires_at_text: item.expires_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_target_views(items: &[RuleTarget]) -> Vec<RuleTargetView> {
    let mut views: Vec<RuleTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_target_view(item));
    }
    views
}

pub fn sum_rule_target_view(view: &RuleTargetView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_version as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_target("code-906", "title-364");
        let view = to_rule_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
