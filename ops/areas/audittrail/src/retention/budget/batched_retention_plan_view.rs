use super::*;

use acme_kit::text;

/// How screens and exports show a batched retention plan.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionPlanView {
    pub code_text: String,
    pub title: String,
    pub grace_days: String,
    pub duration_minutes: String,
    pub limit: String,
    pub name_text: String,
    pub score: String,
    pub margin_text: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_batched_retention_plan_view(item: &BatchedRetentionPlan) -> BatchedRetentionPlanView {
    BatchedRetentionPlanView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        grace_days: item.grace_days.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        limit: item.limit.to_string(),
        name_text: text::truncate(&item.name, 64),
        score: format!("{:.2}", item.score),
        margin_text: format!("{:.2}", item.margin),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_retention_plan_views(items: &[BatchedRetentionPlan]) -> Vec<BatchedRetentionPlanView> {
    let mut views: Vec<BatchedRetentionPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_retention_plan_view(item));
    }
    views
}

pub fn sum_batched_retention_plan_view(view: &BatchedRetentionPlanView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_duration_minutes as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_retention_plan("code-155", "title-241");
        let view = to_batched_retention_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_retention_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
