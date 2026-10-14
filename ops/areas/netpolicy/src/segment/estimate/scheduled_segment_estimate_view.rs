use super::*;

/// How screens and exports show a scheduled segment estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimateView {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub notes: String,
    pub window_days: String,
    pub principal_text: String,
    pub floor_value: String,
    pub visible: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_scheduled_segment_estimate_view(item: &ScheduledSegmentEstimate) -> ScheduledSegmentEstimateView {
    ScheduledSegmentEstimateView {
        code: item.code.clone(),
        name: item.name.clone(),
        unit: item.unit.clone(),
        notes: item.notes.clone(),
        window_days: item.window_days.to_string(),
        principal_text: item.principal.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        visible: format!("{}", item.visible),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_segment_estimate_views(items: &[ScheduledSegmentEstimate]) -> Vec<ScheduledSegmentEstimateView> {
    let mut views: Vec<ScheduledSegmentEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_segment_estimate_view(item));
    }
    views
}

pub fn sum_scheduled_segment_estimate_view(view: &ScheduledSegmentEstimateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_floor_value + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_segment_estimate("code-539", "name-561");
        let view = to_scheduled_segment_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_segment_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
