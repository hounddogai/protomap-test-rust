use super::*;

use acme_kit::text;

/// How screens and exports show a approval snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSnapshotView {
    pub code: String,
    pub label: String,
    pub burst: String,
    pub phone: String,
    pub source_text: String,
    pub key_id: String,
    pub retries: String,
    pub threshold: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_approval_snapshot_view(item: &ApprovalSnapshot) -> ApprovalSnapshotView {
    ApprovalSnapshotView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 40),
        burst: item.burst.to_string(),
        phone: item.phone.clone(),
        source_text: item.source.clone(),
        key_id: item.key_id.clone(),
        retries: item.retries.to_string(),
        threshold: format!("{:.2}", item.threshold),
        notes_text: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_snapshot_views(items: &[ApprovalSnapshot]) -> Vec<ApprovalSnapshotView> {
    let mut views: Vec<ApprovalSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_snapshot_view(item));
    }
    views
}

pub fn sum_approval_snapshot_view(view: &ApprovalSnapshotView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_retries as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_snapshot("code-786", "label-333");
        let view = to_approval_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
