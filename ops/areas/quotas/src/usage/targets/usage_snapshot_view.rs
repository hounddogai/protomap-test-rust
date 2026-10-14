use super::*;

use acme_kit::text;

/// How screens and exports show a usage snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshotView {
    pub code: String,
    pub title_text: String,
    pub reviewer_text: String,
    pub duration_minutes: String,
    pub quota_units_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_usage_snapshot_view(item: &UsageSnapshot) -> UsageSnapshotView {
    UsageSnapshotView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        reviewer_text: item.reviewer.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        quota_units_text: item.quota_units.to_string(),
        source: text::truncate(&item.source, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_snapshot_views(items: &[UsageSnapshot]) -> Vec<UsageSnapshotView> {
    let mut views: Vec<UsageSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_snapshot_view(item));
    }
    views
}

pub fn sum_usage_snapshot_view(view: &UsageSnapshotView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_snapshot("code-372", "title-622");
        let view = to_usage_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
