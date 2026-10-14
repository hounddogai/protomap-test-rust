use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled sink plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkPlanView {
    pub code: String,
    pub title_text: String,
    pub external_ref_text: String,
    pub amount: String,
    pub principal: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_scheduled_sink_plan_view(item: &ScheduledSinkPlan) -> ScheduledSinkPlanView {
    ScheduledSinkPlanView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        external_ref_text: text::truncate(&item.external_ref, 120),
        amount: format!("{:.2}", item.amount),
        principal: item.principal.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_sink_plan_views(items: &[ScheduledSinkPlan]) -> Vec<ScheduledSinkPlanView> {
    let mut views: Vec<ScheduledSinkPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_sink_plan_view(item));
    }
    views
}

pub fn sum_scheduled_sink_plan_view(view: &ScheduledSinkPlanView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_amount + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_sink_plan("code-813", "title-183");
        let view = to_scheduled_sink_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_sink_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
