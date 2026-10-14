use super::*;

use acme_kit::text;

/// How screens and exports show a global timeline adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentView {
    pub code_text: String,
    pub title_text: String,
    pub grace_days: String,
    pub namespace: String,
    pub unit_text: String,
    pub notes: String,
    pub locale: String,
    pub checksum: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_global_timeline_adjustment_view(item: &GlobalTimelineAdjustment) -> GlobalTimelineAdjustmentView {
    GlobalTimelineAdjustmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        unit_text: item.unit.clone(),
        notes: text::truncate(&item.notes, 80),
        locale: text::truncate(&item.locale, 80),
        checksum: item.checksum.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_timeline_adjustment_views(items: &[GlobalTimelineAdjustment]) -> Vec<GlobalTimelineAdjustmentView> {
    let mut views: Vec<GlobalTimelineAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_timeline_adjustment_view(item));
    }
    views
}

pub fn sum_global_timeline_adjustment_view(view: &GlobalTimelineAdjustmentView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_timeline_adjustment("code-812", "title-680");
        let view = to_global_timeline_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_timeline_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
