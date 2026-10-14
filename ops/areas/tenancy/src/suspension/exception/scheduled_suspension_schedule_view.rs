use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled suspension schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionScheduleView {
    pub code_text: String,
    pub label: String,
    pub rotation_days_text: String,
    pub reviewer_text: String,
    pub segment_text: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_scheduled_suspension_schedule_view(item: &ScheduledSuspensionSchedule) -> ScheduledSuspensionScheduleView {
    ScheduledSuspensionScheduleView {
        code_text: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 80),
        rotation_days_text: item.rotation_days.to_string(),
        reviewer_text: item.reviewer.clone(),
        segment_text: item.segment.clone(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_suspension_schedule_views(
    items: &[ScheduledSuspensionSchedule],
) -> Vec<ScheduledSuspensionScheduleView> {
    let mut views: Vec<ScheduledSuspensionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_suspension_schedule_view(item));
    }
    views
}

pub fn sum_scheduled_suspension_schedule_view(view: &ScheduledSuspensionScheduleView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_suspension_schedule("code-838", "label-59");
        let view = to_scheduled_suspension_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_suspension_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
