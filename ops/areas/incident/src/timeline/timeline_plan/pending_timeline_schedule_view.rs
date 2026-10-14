use super::*;

/// How screens and exports show a pending timeline schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleView {
    pub code: String,
    pub name_text: String,
    pub weight: String,
    pub maximum_text: String,
    pub namespace: String,
    pub timezone: String,
    pub updated_at: String,
    pub label: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_pending_timeline_schedule_view(item: &PendingTimelineSchedule) -> PendingTimelineScheduleView {
    PendingTimelineScheduleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        maximum_text: format!("{:.2}", item.maximum),
        namespace: item.namespace.clone(),
        timezone: item.timezone.clone(),
        updated_at: item.updated_at.to_string(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_timeline_schedule_views(items: &[PendingTimelineSchedule]) -> Vec<PendingTimelineScheduleView> {
    let mut views: Vec<PendingTimelineScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_timeline_schedule_view(item));
    }
    views
}

pub fn sum_pending_timeline_schedule_view(view: &PendingTimelineScheduleView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_weight + parsed_maximum + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_timeline_schedule("code-351", "name-448");
        let view = to_pending_timeline_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_timeline_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
