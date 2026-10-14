use super::*;

use acme_kit::text;

/// How screens and exports show a regional canary snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySnapshotView {
    pub code: String,
    pub title: String,
    pub source_text: String,
    pub created_at: String,
    pub scope: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_regional_canary_snapshot_view(item: &RegionalCanarySnapshot) -> RegionalCanarySnapshotView {
    RegionalCanarySnapshotView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        source_text: item.source.clone(),
        created_at: item.created_at.to_string(),
        scope: item.scope.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_canary_snapshot_views(items: &[RegionalCanarySnapshot]) -> Vec<RegionalCanarySnapshotView> {
    let mut views: Vec<RegionalCanarySnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_canary_snapshot_view(item));
    }
    views
}

pub fn sum_regional_canary_snapshot_view(view: &RegionalCanarySnapshotView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_canary_snapshot("code-15", "title-101");
        let view = to_regional_canary_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_canary_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
