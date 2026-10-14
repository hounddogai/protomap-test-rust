use super::*;

use acme_kit::text;

/// How screens and exports show a capped escalation metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetricView {
    pub code: String,
    pub label: String,
    pub rate_text: String,
    pub approved_text: String,
    pub visible_text: String,
    pub floor_value: String,
    pub budget: String,
    pub duration_minutes: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_capped_escalation_metric_view(item: &CappedEscalationMetric) -> CappedEscalationMetricView {
    CappedEscalationMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        rate_text: format!("{:.2}", item.rate),
        approved_text: format!("{}", item.approved),
        visible_text: format!("{}", item.visible),
        floor_value: format!("{:.2}", item.floor_value),
        budget: format!("{:.2}", item.budget),
        duration_minutes: item.duration_minutes.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_escalation_metric_views(items: &[CappedEscalationMetric]) -> Vec<CappedEscalationMetricView> {
    let mut views: Vec<CappedEscalationMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_escalation_metric_view(item));
    }
    views
}

pub fn sum_capped_escalation_metric_view(view: &CappedEscalationMetricView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_rate + parsed_floor_value + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_escalation_metric("code-747", "label-81");
        let view = to_capped_escalation_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_escalation_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
