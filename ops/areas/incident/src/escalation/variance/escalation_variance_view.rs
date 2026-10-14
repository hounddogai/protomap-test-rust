use super::*;

use acme_kit::text;

/// How screens and exports show a escalation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationVarianceView {
    pub code_text: String,
    pub label: String,
    pub segment: String,
    pub starts_at: String,
    pub grace_days_text: String,
    pub capacity: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_escalation_variance_view(item: &EscalationVariance) -> EscalationVarianceView {
    EscalationVarianceView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        segment: text::truncate(&item.segment, 120),
        starts_at: item.starts_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        capacity: item.capacity.to_string(),
        region: text::truncate(&item.region, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_variance_views(items: &[EscalationVariance]) -> Vec<EscalationVarianceView> {
    let mut views: Vec<EscalationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_variance_view(item));
    }
    views
}

pub fn sum_escalation_variance_view(view: &EscalationVarianceView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_grace_days as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_variance("code-631", "label-196");
        let view = to_escalation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
