use super::*;

use acme_kit::text;

/// How screens and exports show a default release snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseSnapshotView {
    pub code: String,
    pub name: String,
    pub grace_days: String,
    pub pinned: String,
    pub maximum: String,
    pub margin: String,
    pub capacity_text: String,
    pub visible: String,
    pub cipher: String,
    pub ratio: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_default_release_snapshot_view(item: &DefaultReleaseSnapshot) -> DefaultReleaseSnapshotView {
    DefaultReleaseSnapshotView {
        code: item.code.clone(),
        name: item.name.clone(),
        grace_days: item.grace_days.to_string(),
        pinned: format!("{}", item.pinned),
        maximum: format!("{:.2}", item.maximum),
        margin: format!("{:.2}", item.margin),
        capacity_text: item.capacity.to_string(),
        visible: format!("{}", item.visible),
        cipher: text::truncate(&item.cipher, 120),
        ratio: format!("{:.2}", item.ratio),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_release_snapshot_views(items: &[DefaultReleaseSnapshot]) -> Vec<DefaultReleaseSnapshotView> {
    let mut views: Vec<DefaultReleaseSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_release_snapshot_view(item));
    }
    views
}

pub fn sum_default_release_snapshot_view(view: &DefaultReleaseSnapshotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
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
    Ok(parsed_grace_days as f64 + parsed_maximum + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_release_snapshot("code-621", "name-956");
        let view = to_default_release_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_release_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
