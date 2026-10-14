use super::*;

use acme_kit::text;

/// How screens and exports show a probe request.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequestView {
    pub code: String,
    pub label: String,
    pub score_text: String,
    pub ceiling: String,
    pub cluster: String,
    pub weight_text: String,
    pub reviewer_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_probe_request_view(item: &ProbeRequest) -> ProbeRequestView {
    ProbeRequestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        score_text: format!("{:.2}", item.score),
        ceiling: format!("{:.2}", item.ceiling),
        cluster: item.cluster.clone(),
        weight_text: format!("{:.2}", item.weight),
        reviewer_text: item.reviewer.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_request_views(items: &[ProbeRequest]) -> Vec<ProbeRequestView> {
    let mut views: Vec<ProbeRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_request_view(item));
    }
    views
}

pub fn sum_probe_request_view(view: &ProbeRequestView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_score + parsed_ceiling + parsed_weight)
}
