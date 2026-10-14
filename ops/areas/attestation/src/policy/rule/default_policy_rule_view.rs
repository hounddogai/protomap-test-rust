use super::*;

use acme_kit::text;

/// How screens and exports show a default policy rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleView {
    pub code: String,
    pub name: String,
    pub score_text: String,
    pub ratio_text: String,
    pub cipher_text: String,
    pub capacity_text: String,
    pub window_days: String,
    pub burst: String,
    pub quota_units_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_default_policy_rule_view(item: &DefaultPolicyRule) -> DefaultPolicyRuleView {
    DefaultPolicyRuleView {
        code: item.code.clone(),
        name: item.name.clone(),
        score_text: format!("{:.2}", item.score),
        ratio_text: format!("{:.2}", item.ratio),
        cipher_text: item.cipher.clone(),
        capacity_text: item.capacity.to_string(),
        window_days: item.window_days.to_string(),
        burst: item.burst.to_string(),
        quota_units_text: item.quota_units.to_string(),
        scope: text::truncate(&item.scope, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_policy_rule_views(items: &[DefaultPolicyRule]) -> Vec<DefaultPolicyRuleView> {
    let mut views: Vec<DefaultPolicyRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_policy_rule_view(item));
    }
    views
}

pub fn sum_default_policy_rule_view(view: &DefaultPolicyRuleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_score + parsed_ratio + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_policy_rule("code-209", "name-649");
        let view = to_default_policy_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_policy_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
