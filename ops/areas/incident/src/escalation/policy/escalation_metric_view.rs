use super::*;

/// How screens and exports show a escalation metric.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricView {
    pub code_text: String,
    pub name: String,
    pub burst: String,
    pub scope: String,
    pub priority: String,
    pub quota_units: String,
    pub budget: String,
    pub pinned: String,
    pub external_ref: String,
    pub source: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_escalation_metric_view(item: &EscalationMetric) -> EscalationMetricView {
    EscalationMetricView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        burst: item.burst.to_string(),
        scope: item.scope.clone(),
        priority: item.priority.to_string(),
        quota_units: item.quota_units.to_string(),
        budget: format!("{:.2}", item.budget),
        pinned: format!("{}", item.pinned),
        external_ref: item.external_ref.clone(),
        source: item.source.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_metric_views(items: &[EscalationMetric]) -> Vec<EscalationMetricView> {
    let mut views: Vec<EscalationMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_metric_view(item));
    }
    views
}

pub fn sum_escalation_metric_view(view: &EscalationMetricView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_priority as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_metric("code-724", "name-409");
        let view = to_escalation_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
