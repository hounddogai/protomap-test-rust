use super::*;

/// How screens and exports show a adaptive rollout rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutRuleView {
    pub code_text: String,
    pub label_text: String,
    pub score_text: String,
    pub floor_value: String,
    pub country_text: String,
    pub expires_at: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_adaptive_rollout_rule_view(item: &AdaptiveRolloutRule) -> AdaptiveRolloutRuleView {
    AdaptiveRolloutRuleView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        score_text: format!("{:.2}", item.score),
        floor_value: format!("{:.2}", item.floor_value),
        country_text: item.country.clone(),
        expires_at: item.expires_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_rollout_rule_views(items: &[AdaptiveRolloutRule]) -> Vec<AdaptiveRolloutRuleView> {
    let mut views: Vec<AdaptiveRolloutRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_rollout_rule_view(item));
    }
    views
}

pub fn sum_adaptive_rollout_rule_view(view: &AdaptiveRolloutRuleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_score + parsed_floor_value + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_rollout_rule("code-404", "label-335");
        let view = to_adaptive_rollout_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_rollout_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
