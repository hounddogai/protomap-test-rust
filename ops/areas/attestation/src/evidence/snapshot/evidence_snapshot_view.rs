use super::*;

use acme_kit::text;

/// How screens and exports show a evidence snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshotView {
    pub code: String,
    pub title: String,
    pub reviewer_text: String,
    pub grace_days: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_evidence_snapshot_view(item: &EvidenceSnapshot) -> EvidenceSnapshotView {
    EvidenceSnapshotView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        reviewer_text: item.reviewer.clone(),
        grace_days: item.grace_days.to_string(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_snapshot_views(items: &[EvidenceSnapshot]) -> Vec<EvidenceSnapshotView> {
    let mut views: Vec<EvidenceSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_snapshot_view(item));
    }
    views
}

pub fn sum_evidence_snapshot_view(view: &EvidenceSnapshotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_snapshot("code-289", "title-701");
        let view = to_evidence_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
