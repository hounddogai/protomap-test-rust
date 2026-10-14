use super::*;

/// How screens and exports show a strict postmortem plan.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPostmortemPlanView {
    pub code_text: String,
    pub label: String,
    pub visible: String,
    pub ttl_seconds_text: String,
    pub active: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_strict_postmortem_plan_view(item: &StrictPostmortemPlan) -> StrictPostmortemPlanView {
    StrictPostmortemPlanView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        active: format!("{}", item.active),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_postmortem_plan_views(items: &[StrictPostmortemPlan]) -> Vec<StrictPostmortemPlanView> {
    let mut views: Vec<StrictPostmortemPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_postmortem_plan_view(item));
    }
    views
}

pub fn sum_strict_postmortem_plan_view(view: &StrictPostmortemPlanView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_postmortem_plan("code-284", "label-739");
        let view = to_strict_postmortem_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_postmortem_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
