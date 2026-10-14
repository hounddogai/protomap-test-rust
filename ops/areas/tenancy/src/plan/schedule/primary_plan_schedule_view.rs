use super::*;

use acme_kit::text;

/// How screens and exports show a primary plan schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanScheduleView {
    pub code_text: String,
    pub name: String,
    pub quota_units: String,
    pub ttl_seconds: String,
    pub grace_days: String,
    pub description: String,
    pub priority_text: String,
    pub tenant_id: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_primary_plan_schedule_view(item: &PrimaryPlanSchedule) -> PrimaryPlanScheduleView {
    PrimaryPlanScheduleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        grace_days: item.grace_days.to_string(),
        description: text::truncate(&item.description, 64),
        priority_text: item.priority.to_string(),
        tenant_id: item.tenant_id.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_plan_schedule_views(items: &[PrimaryPlanSchedule]) -> Vec<PrimaryPlanScheduleView> {
    let mut views: Vec<PrimaryPlanScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_plan_schedule_view(item));
    }
    views
}

pub fn sum_primary_plan_schedule_view(view: &PrimaryPlanScheduleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_ttl_seconds as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_plan_schedule("code-598", "name-593");
        let view = to_primary_plan_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_plan_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
