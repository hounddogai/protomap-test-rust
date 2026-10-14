use super::*;

use acme_kit::text;

/// How screens and exports show a overage snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageSnapshotView {
    pub code: String,
    pub title: String,
    pub expires_at_text: String,
    pub ends_at: String,
    pub scope: String,
    pub threshold: String,
    pub phone: String,
    pub amount: String,
    pub label: String,
    pub summary: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_overage_snapshot_view(item: &OverageSnapshot) -> OverageSnapshotView {
    OverageSnapshotView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        expires_at_text: item.expires_at.to_string(),
        ends_at: item.ends_at.to_string(),
        scope: item.scope.clone(),
        threshold: format!("{:.2}", item.threshold),
        phone: text::truncate(&item.phone, 40),
        amount: format!("{:.2}", item.amount),
        label: item.label.clone(),
        summary: text::truncate(&item.summary, 40),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_snapshot_views(items: &[OverageSnapshot]) -> Vec<OverageSnapshotView> {
    let mut views: Vec<OverageSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_snapshot_view(item));
    }
    views
}

pub fn sum_overage_snapshot_view(view: &OverageSnapshotView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_ends_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_snapshot("code-824", "title-376");
        let view = to_overage_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
