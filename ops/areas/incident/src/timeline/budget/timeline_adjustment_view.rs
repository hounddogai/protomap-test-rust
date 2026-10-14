use super::*;

use acme_kit::text;

/// How screens and exports show a timeline adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAdjustmentView {
    pub code: String,
    pub name: String,
    pub expires_at_text: String,
    pub updated_at_text: String,
    pub visible: String,
    pub quota_units: String,
    pub ratio_text: String,
    pub owner_id: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_timeline_adjustment_view(item: &TimelineAdjustment) -> TimelineAdjustmentView {
    TimelineAdjustmentView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 80),
        expires_at_text: item.expires_at.to_string(),
        updated_at_text: item.updated_at.to_string(),
        visible: format!("{}", item.visible),
        quota_units: item.quota_units.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        owner_id: text::truncate(&item.owner_id, 64),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_adjustment_views(items: &[TimelineAdjustment]) -> Vec<TimelineAdjustmentView> {
    let mut views: Vec<TimelineAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_adjustment_view(item));
    }
    views
}

pub fn sum_timeline_adjustment_view(view: &TimelineAdjustmentView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_updated_at as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_adjustment("code-83", "name-714");
        let view = to_timeline_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
