use super::*;

use acme_kit::text;

/// How screens and exports show a fallback postmortem metric.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemMetricView {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub ttl_seconds: String,
    pub approved: String,
    pub duration_minutes: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_fallback_postmortem_metric_view(item: &FallbackPostmortemMetric) -> FallbackPostmortemMetricView {
    FallbackPostmortemMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        principal: text::truncate(&item.principal, 64),
        ttl_seconds: item.ttl_seconds.to_string(),
        approved: format!("{}", item.approved),
        duration_minutes: item.duration_minutes.to_string(),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_postmortem_metric_views(items: &[FallbackPostmortemMetric]) -> Vec<FallbackPostmortemMetricView> {
    let mut views: Vec<FallbackPostmortemMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_postmortem_metric_view(item));
    }
    views
}

pub fn sum_fallback_postmortem_metric_view(view: &FallbackPostmortemMetricView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_postmortem_metric("code-698", "name-945");
        let view = to_fallback_postmortem_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_postmortem_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
