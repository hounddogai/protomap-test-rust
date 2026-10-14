use super::*;

use acme_kit::text;

/// How screens and exports show a timeline milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneView {
    pub code: String,
    pub title: String,
    pub quantity: String,
    pub duration_minutes_text: String,
    pub notes: String,
    pub owner_id: String,
    pub segment: String,
    pub quota_units: String,
    pub source: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_timeline_milestone_view(item: &TimelineMilestone) -> TimelineMilestoneView {
    TimelineMilestoneView {
        code: item.code.clone(),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        notes: text::truncate(&item.notes, 120),
        owner_id: text::truncate(&item.owner_id, 120),
        segment: text::truncate(&item.segment, 120),
        quota_units: item.quota_units.to_string(),
        source: item.source.clone(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_milestone_views(items: &[TimelineMilestone]) -> Vec<TimelineMilestoneView> {
    let mut views: Vec<TimelineMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_milestone_view(item));
    }
    views
}

pub fn sum_timeline_milestone_view(view: &TimelineMilestoneView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_duration_minutes as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_milestone("code-812", "title-514");
        let view = to_timeline_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
