use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled timeline baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaselineView {
    pub code: String,
    pub label: String,
    pub channel_text: String,
    pub archived: String,
    pub source: String,
    pub country: String,
    pub namespace: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_scheduled_timeline_baseline_view(item: &ScheduledTimelineBaseline) -> ScheduledTimelineBaselineView {
    ScheduledTimelineBaselineView {
        code: item.code.clone(),
        label: item.label.clone(),
        channel_text: text::truncate(&item.channel, 80),
        archived: format!("{}", item.archived),
        source: text::truncate(&item.source, 64),
        country: item.country.clone(),
        namespace: item.namespace.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_timeline_baseline_views(items: &[ScheduledTimelineBaseline]) -> Vec<ScheduledTimelineBaselineView> {
    let mut views: Vec<ScheduledTimelineBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_timeline_baseline_view(item));
    }
    views
}

pub fn sum_scheduled_timeline_baseline_view(view: &ScheduledTimelineBaselineView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_timeline_baseline("code-407", "label-358");
        let view = to_scheduled_timeline_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_timeline_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
