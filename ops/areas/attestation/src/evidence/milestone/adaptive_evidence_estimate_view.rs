use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive evidence estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimateView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub margin: String,
    pub source_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_adaptive_evidence_estimate_view(item: &AdaptiveEvidenceEstimate) -> AdaptiveEvidenceEstimateView {
    AdaptiveEvidenceEstimateView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        margin: format!("{:.2}", item.margin),
        source_text: text::truncate(&item.source, 120),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_evidence_estimate_views(items: &[AdaptiveEvidenceEstimate]) -> Vec<AdaptiveEvidenceEstimateView> {
    let mut views: Vec<AdaptiveEvidenceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_evidence_estimate_view(item));
    }
    views
}

pub fn sum_adaptive_evidence_estimate_view(view: &AdaptiveEvidenceEstimateView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_score + parsed_margin + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_evidence_estimate("code-273", "label-838");
        let view = to_adaptive_evidence_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_evidence_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
