use super::*;

/// How screens and exports show a rollback estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEstimateView {
    pub code_text: String,
    pub label: String,
    pub burst: String,
    pub archived: String,
    pub cluster: String,
    pub active_text: String,
    pub owner_id: String,
    pub attempts: String,
    pub locale_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_rollback_estimate_view(item: &RollbackEstimate) -> RollbackEstimateView {
    RollbackEstimateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        archived: format!("{}", item.archived),
        cluster: item.cluster.clone(),
        active_text: format!("{}", item.active),
        owner_id: item.owner_id.clone(),
        attempts: item.attempts.to_string(),
        locale_text: item.locale.clone(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_estimate_views(items: &[RollbackEstimate]) -> Vec<RollbackEstimateView> {
    let mut views: Vec<RollbackEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_estimate_view(item));
    }
    views
}

pub fn sum_rollback_estimate_view(view: &RollbackEstimateView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_attempts as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_estimate("code-977", "label-511");
        let view = to_rollback_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
