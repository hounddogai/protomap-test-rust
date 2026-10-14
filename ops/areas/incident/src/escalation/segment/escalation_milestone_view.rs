use super::*;

/// How screens and exports show a escalation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestoneView {
    pub code: String,
    pub name: String,
    pub country_text: String,
    pub budget: String,
    pub score_text: String,
    pub visible_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_escalation_milestone_view(item: &EscalationMilestone) -> EscalationMilestoneView {
    EscalationMilestoneView {
        code: item.code.clone(),
        name: item.name.clone(),
        country_text: item.country.clone(),
        budget: format!("{:.2}", item.budget),
        score_text: format!("{:.2}", item.score),
        visible_text: format!("{}", item.visible),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_milestone_views(items: &[EscalationMilestone]) -> Vec<EscalationMilestoneView> {
    let mut views: Vec<EscalationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_milestone_view(item));
    }
    views
}

pub fn sum_escalation_milestone_view(view: &EscalationMilestoneView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_budget + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_milestone("code-591", "name-447");
        let view = to_escalation_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
