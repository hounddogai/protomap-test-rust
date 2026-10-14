use super::*;

use acme_kit::text;

/// How screens and exports show a default zone snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSnapshotView {
    pub code: String,
    pub title: String,
    pub updated_at: String,
    pub visible_text: String,
    pub margin: String,
    pub summary_text: String,
    pub reviewer: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_default_zone_snapshot_view(item: &DefaultZoneSnapshot) -> DefaultZoneSnapshotView {
    DefaultZoneSnapshotView {
        code: item.code.clone(),
        title: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        visible_text: format!("{}", item.visible),
        margin: format!("{:.2}", item.margin),
        summary_text: item.summary.clone(),
        reviewer: text::truncate(&item.reviewer, 40),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_zone_snapshot_views(items: &[DefaultZoneSnapshot]) -> Vec<DefaultZoneSnapshotView> {
    let mut views: Vec<DefaultZoneSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_zone_snapshot_view(item));
    }
    views
}

pub fn sum_default_zone_snapshot_view(view: &DefaultZoneSnapshotView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_margin + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_zone_snapshot("code-74", "title-948");
        let view = to_default_zone_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_zone_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
