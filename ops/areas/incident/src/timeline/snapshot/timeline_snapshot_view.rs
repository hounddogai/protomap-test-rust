use super::*;

use acme_kit::text;

/// How screens and exports show a timeline snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSnapshotView {
    pub code: String,
    pub title: String,
    pub discount: String,
    pub maximum: String,
    pub namespace: String,
    pub category: String,
    pub margin: String,
    pub active_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_timeline_snapshot_view(item: &TimelineSnapshot) -> TimelineSnapshotView {
    TimelineSnapshotView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        discount: format!("{:.2}", item.discount),
        maximum: format!("{:.2}", item.maximum),
        namespace: item.namespace.clone(),
        category: item.category.clone(),
        margin: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_snapshot_views(items: &[TimelineSnapshot]) -> Vec<TimelineSnapshotView> {
    let mut views: Vec<TimelineSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_snapshot_view(item));
    }
    views
}

pub fn sum_timeline_snapshot_view(view: &TimelineSnapshotView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_discount + parsed_maximum + parsed_margin)
}
