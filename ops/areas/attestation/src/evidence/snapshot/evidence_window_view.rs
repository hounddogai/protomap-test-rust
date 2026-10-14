use super::*;

use acme_kit::text;

/// How screens and exports show a evidence window.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowView {
    pub code: String,
    pub label_text: String,
    pub scope: String,
    pub unit: String,
    pub title: String,
    pub archived: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_evidence_window_view(item: &EvidenceWindow) -> EvidenceWindowView {
    EvidenceWindowView {
        code: text::truncate(&item.code, 40),
        label_text: text::truncate(&item.label, 40),
        scope: text::truncate(&item.scope, 64),
        unit: text::truncate(&item.unit, 80),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_window_views(items: &[EvidenceWindow]) -> Vec<EvidenceWindowView> {
    let mut views: Vec<EvidenceWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_window_view(item));
    }
    views
}

pub fn sum_evidence_window_view(view: &EvidenceWindowView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_window("code-753", "label-408");
        let view = to_evidence_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
