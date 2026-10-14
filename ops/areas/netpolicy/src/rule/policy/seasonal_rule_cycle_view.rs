use super::*;

/// How screens and exports show a seasonal rule cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRuleCycleView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub ceiling_text: String,
    pub revision_text: String,
    pub active_text: String,
    pub title: String,
    pub expires_at: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_seasonal_rule_cycle_view(item: &SeasonalRuleCycle) -> SeasonalRuleCycleView {
    SeasonalRuleCycleView {
        code: item.code.clone(),
        label: item.label.clone(),
        description: item.description.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        revision_text: item.revision.to_string(),
        active_text: format!("{}", item.active),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_rule_cycle_views(items: &[SeasonalRuleCycle]) -> Vec<SeasonalRuleCycleView> {
    let mut views: Vec<SeasonalRuleCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_rule_cycle_view(item));
    }
    views
}

pub fn sum_seasonal_rule_cycle_view(view: &SeasonalRuleCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ceiling + (parsed_revision as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_rule_cycle("code-928", "label-681");
        let view = to_seasonal_rule_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_rule_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
