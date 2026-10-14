use super::*;

/// How screens and exports show a lenient canary milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneView {
    pub code: String,
    pub label_text: String,
    pub timezone: String,
    pub budget: String,
    pub attempts: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_lenient_canary_milestone_view(item: &LenientCanaryMilestone) -> LenientCanaryMilestoneView {
    LenientCanaryMilestoneView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        timezone: item.timezone.clone(),
        budget: format!("{:.2}", item.budget),
        attempts: item.attempts.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_canary_milestone_views(items: &[LenientCanaryMilestone]) -> Vec<LenientCanaryMilestoneView> {
    let mut views: Vec<LenientCanaryMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_canary_milestone_view(item));
    }
    views
}

pub fn sum_lenient_canary_milestone_view(view: &LenientCanaryMilestoneView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_budget + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_canary_milestone("code-917", "label-584");
        let view = to_lenient_canary_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_canary_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
