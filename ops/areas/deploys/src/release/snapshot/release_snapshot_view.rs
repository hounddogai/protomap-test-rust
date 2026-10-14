use super::*;

use acme_kit::text;

/// How screens and exports show a release snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSnapshotView {
    pub code: String,
    pub title: String,
    pub quota_units: String,
    pub namespace: String,
    pub maximum: String,
    pub region: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_release_snapshot_view(item: &ReleaseSnapshot) -> ReleaseSnapshotView {
    ReleaseSnapshotView {
        code: item.code.clone(),
        title: item.title.clone(),
        quota_units: item.quota_units.to_string(),
        namespace: text::truncate(&item.namespace, 120),
        maximum: format!("{:.2}", item.maximum),
        region: item.region.clone(),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_snapshot_views(items: &[ReleaseSnapshot]) -> Vec<ReleaseSnapshotView> {
    let mut views: Vec<ReleaseSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_snapshot_view(item));
    }
    views
}

pub fn sum_release_snapshot_view(view: &ReleaseSnapshotView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_snapshot("code-897", "title-157");
        let view = to_release_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
