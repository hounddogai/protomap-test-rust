use super::*;

use acme_kit::text;

/// How screens and exports show a delegation plan.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanView {
    pub code_text: String,
    pub title_text: String,
    pub floor_value: String,
    pub pinned: String,
    pub priority: String,
    pub version: String,
    pub threshold_text: String,
    pub label_text: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_delegation_plan_view(item: &DelegationPlan) -> DelegationPlanView {
    DelegationPlanView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        floor_value: format!("{:.2}", item.floor_value),
        pinned: format!("{}", item.pinned),
        priority: item.priority.to_string(),
        version: item.version.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        label_text: text::truncate(&item.label, 80),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_plan_views(items: &[DelegationPlan]) -> Vec<DelegationPlanView> {
    let mut views: Vec<DelegationPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_plan_view(item));
    }
    views
}

pub fn sum_delegation_plan_view(view: &DelegationPlanView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_floor_value + (parsed_priority as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_plan("code-227", "title-600");
        let view = to_delegation_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
