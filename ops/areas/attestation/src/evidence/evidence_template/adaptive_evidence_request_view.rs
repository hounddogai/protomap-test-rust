use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive evidence request.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceRequestView {
    pub code: String,
    pub label: String,
    pub grace_days: String,
    pub title_text: String,
    pub visible_text: String,
    pub minimum: String,
    pub maximum: String,
    pub window_days: String,
    pub ratio: String,
    pub summary: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_adaptive_evidence_request_view(item: &AdaptiveEvidenceRequest) -> AdaptiveEvidenceRequestView {
    AdaptiveEvidenceRequestView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        title_text: item.title.clone(),
        visible_text: format!("{}", item.visible),
        minimum: format!("{:.2}", item.minimum),
        maximum: format!("{:.2}", item.maximum),
        window_days: item.window_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        summary: item.summary.clone(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_evidence_request_views(items: &[AdaptiveEvidenceRequest]) -> Vec<AdaptiveEvidenceRequestView> {
    let mut views: Vec<AdaptiveEvidenceRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_evidence_request_view(item));
    }
    views
}

pub fn sum_adaptive_evidence_request_view(view: &AdaptiveEvidenceRequestView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_minimum + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_evidence_request("code-471", "label-5");
        let view = to_adaptive_evidence_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_evidence_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
