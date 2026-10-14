use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive probe draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraftView {
    pub code: String,
    pub label: String,
    pub sequence: String,
    pub amount: String,
    pub score: String,
    pub issuer: String,
    pub ends_at: String,
    pub quota_units_text: String,
    pub minimum: String,
    pub scope: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_adaptive_probe_draft_view(item: &AdaptiveProbeDraft) -> AdaptiveProbeDraftView {
    AdaptiveProbeDraftView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        sequence: item.sequence.to_string(),
        amount: format!("{:.2}", item.amount),
        score: format!("{:.2}", item.score),
        issuer: item.issuer.clone(),
        ends_at: item.ends_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        minimum: format!("{:.2}", item.minimum),
        scope: item.scope.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_probe_draft_views(items: &[AdaptiveProbeDraft]) -> Vec<AdaptiveProbeDraftView> {
    let mut views: Vec<AdaptiveProbeDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_probe_draft_view(item));
    }
    views
}

pub fn sum_adaptive_probe_draft_view(view: &AdaptiveProbeDraftView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_amount + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_probe_draft("code-132", "label-698");
        let view = to_adaptive_probe_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_probe_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
