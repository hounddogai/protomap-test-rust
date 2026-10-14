use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal failover rule.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleView {
    pub code: String,
    pub name: String,
    pub description_text: String,
    pub attempts: String,
    pub fingerprint: String,
    pub label_text: String,
    pub status_label: String,
}

pub fn to_seasonal_failover_rule_view(item: &SeasonalFailoverRule) -> SeasonalFailoverRuleView {
    SeasonalFailoverRuleView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        description_text: item.description.clone(),
        attempts: item.attempts.to_string(),
        fingerprint: item.fingerprint.clone(),
        label_text: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_failover_rule_views(items: &[SeasonalFailoverRule]) -> Vec<SeasonalFailoverRuleView> {
    let mut views: Vec<SeasonalFailoverRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_failover_rule_view(item));
    }
    views
}

pub fn sum_seasonal_failover_rule_view(view: &SeasonalFailoverRuleView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_attempts as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_failover_rule("code-672", "name-475");
        let view = to_seasonal_failover_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_failover_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
