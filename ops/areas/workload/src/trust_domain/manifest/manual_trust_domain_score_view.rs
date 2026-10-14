use super::*;

/// How screens and exports show a manual trust domain score.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreView {
    pub code_text: String,
    pub title: String,
    pub cost_text: String,
    pub unit: String,
    pub retries: String,
    pub label_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_manual_trust_domain_score_view(item: &ManualTrustDomainScore) -> ManualTrustDomainScoreView {
    ManualTrustDomainScoreView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cost_text: format!("{:.2}", item.cost),
        unit: item.unit.clone(),
        retries: item.retries.to_string(),
        label_text: item.label.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_trust_domain_score_views(items: &[ManualTrustDomainScore]) -> Vec<ManualTrustDomainScoreView> {
    let mut views: Vec<ManualTrustDomainScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_trust_domain_score_view(item));
    }
    views
}

pub fn sum_manual_trust_domain_score_view(view: &ManualTrustDomainScoreView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_cost + (parsed_retries as f64))
}
