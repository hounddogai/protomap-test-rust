use super::*;

use acme_kit::text;

/// How screens and exports show a strict segment entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntryView {
    pub code: String,
    pub title: String,
    pub approved: String,
    pub phone: String,
    pub source: String,
    pub archived: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_strict_segment_entry_view(item: &StrictSegmentEntry) -> StrictSegmentEntryView {
    StrictSegmentEntryView {
        code: item.code.clone(),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        phone: item.phone.clone(),
        source: text::truncate(&item.source, 64),
        archived: format!("{}", item.archived),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_segment_entry_views(items: &[StrictSegmentEntry]) -> Vec<StrictSegmentEntryView> {
    let mut views: Vec<StrictSegmentEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_segment_entry_view(item));
    }
    views
}

pub fn sum_strict_segment_entry_view(view: &StrictSegmentEntryView) -> Result<f64, String> {
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
        let item = new_strict_segment_entry("code-745", "title-896");
        let view = to_strict_segment_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_segment_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
