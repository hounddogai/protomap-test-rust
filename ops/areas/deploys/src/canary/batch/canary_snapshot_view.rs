use super::*;

use acme_kit::text;

/// How screens and exports show a canary snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotView {
    pub code: String,
    pub title_text: String,
    pub ratio: String,
    pub country_text: String,
    pub locale: String,
    pub created_at: String,
    pub fingerprint: String,
    pub checksum: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_canary_snapshot_view(item: &CanarySnapshot) -> CanarySnapshotView {
    CanarySnapshotView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        ratio: format!("{:.2}", item.ratio),
        country_text: text::truncate(&item.country, 80),
        locale: item.locale.clone(),
        created_at: item.created_at.to_string(),
        fingerprint: item.fingerprint.clone(),
        checksum: item.checksum.clone(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_snapshot_views(items: &[CanarySnapshot]) -> Vec<CanarySnapshotView> {
    let mut views: Vec<CanarySnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_snapshot_view(item));
    }
    views
}

pub fn sum_canary_snapshot_view(view: &CanarySnapshotView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ratio + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_snapshot("code-873", "title-816");
        let view = to_canary_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
