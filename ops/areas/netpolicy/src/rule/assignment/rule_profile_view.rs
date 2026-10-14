use super::*;

/// How screens and exports show a rule profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfileView {
    pub code: String,
    pub name_text: String,
    pub maximum_text: String,
    pub attempts: String,
    pub retries: String,
    pub scope: String,
    pub summary_text: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_rule_profile_view(item: &RuleProfile) -> RuleProfileView {
    RuleProfileView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        attempts: item.attempts.to_string(),
        retries: item.retries.to_string(),
        scope: item.scope.clone(),
        summary_text: item.summary.clone(),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_profile_views(items: &[RuleProfile]) -> Vec<RuleProfileView> {
    let mut views: Vec<RuleProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_profile_view(item));
    }
    views
}

pub fn sum_rule_profile_view(view: &RuleProfileView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_maximum + (parsed_attempts as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_profile("code-997", "name-515");
        let view = to_rule_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
