use super::*;

/// How screens and exports show a legacy escalation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEscalationMilestoneView {
    pub code: String,
    pub name_text: String,
    pub window_days_text: String,
    pub limit: String,
    pub reviewer: String,
    pub key_id: String,
    pub starts_at: String,
    pub rotation_days: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_legacy_escalation_milestone_view(item: &LegacyEscalationMilestone) -> LegacyEscalationMilestoneView {
    LegacyEscalationMilestoneView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        window_days_text: item.window_days.to_string(),
        limit: item.limit.to_string(),
        reviewer: item.reviewer.clone(),
        key_id: item.key_id.clone(),
        starts_at: item.starts_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_escalation_milestone_views(items: &[LegacyEscalationMilestone]) -> Vec<LegacyEscalationMilestoneView> {
    let mut views: Vec<LegacyEscalationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_escalation_milestone_view(item));
    }
    views
}

pub fn sum_legacy_escalation_milestone_view(view: &LegacyEscalationMilestoneView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_limit as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_escalation_milestone("code-146", "name-465");
        let view = to_legacy_escalation_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_escalation_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
