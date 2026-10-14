use super::*;

/// How screens and exports show a global plan schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleView {
    pub code: String,
    pub label: String,
    pub threshold: String,
    pub starts_at: String,
    pub expires_at_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_global_plan_schedule_view(item: &GlobalPlanSchedule) -> GlobalPlanScheduleView {
    GlobalPlanScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        starts_at: item.starts_at.to_string(),
        expires_at_text: item.expires_at.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_plan_schedule_views(items: &[GlobalPlanSchedule]) -> Vec<GlobalPlanScheduleView> {
    let mut views: Vec<GlobalPlanScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_plan_schedule_view(item));
    }
    views
}

pub fn sum_global_plan_schedule_view(view: &GlobalPlanScheduleView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_threshold + (parsed_starts_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_plan_schedule("code-108", "label-927");
        let view = to_global_plan_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_plan_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
