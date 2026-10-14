use super::*;

use acme_kit::text;

/// How screens and exports show a pending token metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTokenMetricView {
    pub code: String,
    pub name_text: String,
    pub score: String,
    pub contact_email: String,
    pub priority: String,
    pub cipher: String,
    pub ttl_seconds_text: String,
    pub updated_at: String,
    pub issuer_text: String,
    pub checksum: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_pending_token_metric_view(item: &PendingTokenMetric) -> PendingTokenMetricView {
    PendingTokenMetricView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        score: format!("{:.2}", item.score),
        contact_email: text::truncate(&item.contact_email, 64),
        priority: item.priority.to_string(),
        cipher: item.cipher.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        updated_at: item.updated_at.to_string(),
        issuer_text: item.issuer.clone(),
        checksum: item.checksum.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_token_metric_views(items: &[PendingTokenMetric]) -> Vec<PendingTokenMetricView> {
    let mut views: Vec<PendingTokenMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_token_metric_view(item));
    }
    views
}

pub fn sum_pending_token_metric_view(view: &PendingTokenMetricView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_score + (parsed_priority as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_token_metric("code-578", "name-299");
        let view = to_pending_token_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_token_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
