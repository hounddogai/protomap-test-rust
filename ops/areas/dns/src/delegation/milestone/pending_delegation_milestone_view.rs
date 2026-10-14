use super::*;

/// How screens and exports show a pending delegation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationMilestoneView {
    pub code_text: String,
    pub title_text: String,
    pub score: String,
    pub cost: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_pending_delegation_milestone_view(item: &PendingDelegationMilestone) -> PendingDelegationMilestoneView {
    PendingDelegationMilestoneView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        score: format!("{:.2}", item.score),
        cost: format!("{:.2}", item.cost),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_delegation_milestone_views(
    items: &[PendingDelegationMilestone],
) -> Vec<PendingDelegationMilestoneView> {
    let mut views: Vec<PendingDelegationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_delegation_milestone_view(item));
    }
    views
}

pub fn sum_pending_delegation_milestone_view(view: &PendingDelegationMilestoneView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_score + parsed_cost + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_delegation_milestone("code-28", "title-440");
        let view = to_pending_delegation_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_delegation_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
