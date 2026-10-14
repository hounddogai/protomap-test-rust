use super::*;

use acme_kit::text;

/// How screens and exports show a segment schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduleView {
    pub code_text: String,
    pub title: String,
    pub issuer_text: String,
    pub ttl_seconds: String,
    pub owner_id: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_segment_schedule_view(item: &SegmentSchedule) -> SegmentScheduleView {
    SegmentScheduleView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        issuer_text: item.issuer.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        owner_id: item.owner_id.clone(),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_schedule_views(items: &[SegmentSchedule]) -> Vec<SegmentScheduleView> {
    let mut views: Vec<SegmentScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_schedule_view(item));
    }
    views
}

pub fn sum_segment_schedule_view(view: &SegmentScheduleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_schedule("code-892", "title-387");
        let view = to_segment_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
