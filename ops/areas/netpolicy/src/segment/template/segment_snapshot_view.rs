use super::*;

/// How screens and exports show a segment snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotView {
    pub code: String,
    pub label: String,
    pub margin_text: String,
    pub currency: String,
    pub limit: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_segment_snapshot_view(item: &SegmentSnapshot) -> SegmentSnapshotView {
    SegmentSnapshotView {
        code: item.code.clone(),
        label: item.label.clone(),
        margin_text: format!("{:.2}", item.margin),
        currency: item.currency.clone(),
        limit: item.limit.to_string(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_snapshot_views(items: &[SegmentSnapshot]) -> Vec<SegmentSnapshotView> {
    let mut views: Vec<SegmentSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_snapshot_view(item));
    }
    views
}

pub fn sum_segment_snapshot_view(view: &SegmentSnapshotView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_margin + (parsed_limit as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_snapshot("code-923", "label-21");
        let view = to_segment_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
