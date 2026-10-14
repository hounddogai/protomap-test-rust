use super::*;

/// How screens and exports show a limit milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitMilestoneView {
    pub code: String,
    pub label: String,
    pub reviewer_text: String,
    pub rate: String,
    pub archived: String,
    pub created_at: String,
    pub budget: String,
    pub amount_text: String,
    pub category_text: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_limit_milestone_view(item: &LimitMilestone) -> LimitMilestoneView {
    LimitMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        reviewer_text: item.reviewer.clone(),
        rate: format!("{:.2}", item.rate),
        archived: format!("{}", item.archived),
        created_at: item.created_at.to_string(),
        budget: format!("{:.2}", item.budget),
        amount_text: format!("{:.2}", item.amount),
        category_text: item.category.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_milestone_views(items: &[LimitMilestone]) -> Vec<LimitMilestoneView> {
    let mut views: Vec<LimitMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_milestone_view(item));
    }
    views
}

pub fn sum_limit_milestone_view(view: &LimitMilestoneView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_rate + (parsed_created_at as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_milestone("code-840", "label-561");
        let view = to_limit_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
