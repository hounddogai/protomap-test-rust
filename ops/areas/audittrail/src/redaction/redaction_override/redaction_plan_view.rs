use super::*;

use acme_kit::text;

/// How screens and exports show a redaction plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlanView {
    pub code: String,
    pub label: String,
    pub budget: String,
    pub window_days_text: String,
    pub rotation_days: String,
    pub checksum: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_redaction_plan_view(item: &RedactionPlan) -> RedactionPlanView {
    RedactionPlanView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        budget: format!("{:.2}", item.budget),
        window_days_text: item.window_days.to_string(),
        rotation_days: item.rotation_days.to_string(),
        checksum: item.checksum.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_plan_views(items: &[RedactionPlan]) -> Vec<RedactionPlanView> {
    let mut views: Vec<RedactionPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_plan_view(item));
    }
    views
}

pub fn sum_redaction_plan_view(view: &RedactionPlanView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_window_days as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_plan("code-417", "label-614");
        let view = to_redaction_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
