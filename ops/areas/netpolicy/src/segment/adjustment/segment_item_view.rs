use super::*;

use acme_kit::text;

/// How screens and exports show a segment item.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentItemView {
    pub code: String,
    pub name: String,
    pub capacity: String,
    pub cluster_text: String,
    pub principal_text: String,
    pub revision: String,
    pub window_days_text: String,
    pub reviewer: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_segment_item_view(item: &SegmentItem) -> SegmentItemView {
    SegmentItemView {
        code: item.code.clone(),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        cluster_text: text::truncate(&item.cluster, 40),
        principal_text: item.principal.clone(),
        revision: item.revision.to_string(),
        window_days_text: item.window_days.to_string(),
        reviewer: item.reviewer.clone(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_item_views(items: &[SegmentItem]) -> Vec<SegmentItemView> {
    let mut views: Vec<SegmentItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_item_view(item));
    }
    views
}

pub fn sum_segment_item_view(view: &SegmentItemView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_revision as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_item("code-472", "name-356");
        let view = to_segment_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
