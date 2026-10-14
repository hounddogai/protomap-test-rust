use super::*;

use acme_kit::text;

/// How screens and exports show a approval metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalMetricView {
    pub code: String,
    pub name: String,
    pub score: String,
    pub revision: String,
    pub reviewer_text: String,
    pub currency: String,
    pub threshold: String,
    pub tenant_id_text: String,
    pub active: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_approval_metric_view(item: &ApprovalMetric) -> ApprovalMetricView {
    ApprovalMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        revision: item.revision.to_string(),
        reviewer_text: item.reviewer.clone(),
        currency: item.currency.clone(),
        threshold: format!("{:.2}", item.threshold),
        tenant_id_text: text::truncate(&item.tenant_id, 64),
        active: format!("{}", item.active),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_metric_views(items: &[ApprovalMetric]) -> Vec<ApprovalMetricView> {
    let mut views: Vec<ApprovalMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_metric_view(item));
    }
    views
}

pub fn sum_approval_metric_view(view: &ApprovalMetricView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_score + (parsed_revision as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_metric("code-572", "name-331");
        let view = to_approval_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
