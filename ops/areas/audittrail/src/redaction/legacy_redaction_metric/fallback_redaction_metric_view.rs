use super::*;

use acme_kit::text;

/// How screens and exports show a fallback redaction metric.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetricView {
    pub code: String,
    pub name: String,
    pub locale_text: String,
    pub score: String,
    pub key_id_text: String,
    pub unit: String,
    pub host_text: String,
    pub attempts: String,
    pub issuer: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_fallback_redaction_metric_view(item: &FallbackRedactionMetric) -> FallbackRedactionMetricView {
    FallbackRedactionMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        locale_text: item.locale.clone(),
        score: format!("{:.2}", item.score),
        key_id_text: item.key_id.clone(),
        unit: item.unit.clone(),
        host_text: text::truncate(&item.host, 40),
        attempts: item.attempts.to_string(),
        issuer: item.issuer.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_redaction_metric_views(items: &[FallbackRedactionMetric]) -> Vec<FallbackRedactionMetricView> {
    let mut views: Vec<FallbackRedactionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_redaction_metric_view(item));
    }
    views
}

pub fn sum_fallback_redaction_metric_view(view: &FallbackRedactionMetricView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_score + (parsed_attempts as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_redaction_metric("code-504", "name-466");
        let view = to_fallback_redaction_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_redaction_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
