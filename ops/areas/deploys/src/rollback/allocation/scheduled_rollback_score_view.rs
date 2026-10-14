use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled rollback score.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRollbackScoreView {
    pub code_text: String,
    pub label: String,
    pub capacity_text: String,
    pub fingerprint: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_scheduled_rollback_score_view(item: &ScheduledRollbackScore) -> ScheduledRollbackScoreView {
    ScheduledRollbackScoreView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rollback_score_views(items: &[ScheduledRollbackScore]) -> Vec<ScheduledRollbackScoreView> {
    let mut views: Vec<ScheduledRollbackScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rollback_score_view(item));
    }
    views
}

pub fn sum_scheduled_rollback_score_view(view: &ScheduledRollbackScoreView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_rollback_score("code-439", "label-496");
        let view = to_scheduled_rollback_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_rollback_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
