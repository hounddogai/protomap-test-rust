use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled overage estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageEstimateView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub source: String,
    pub attempts: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_scheduled_overage_estimate_view(item: &ScheduledOverageEstimate) -> ScheduledOverageEstimateView {
    ScheduledOverageEstimateView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        discount: format!("{:.2}", item.discount),
        source: item.source.clone(),
        attempts: item.attempts.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_overage_estimate_views(items: &[ScheduledOverageEstimate]) -> Vec<ScheduledOverageEstimateView> {
    let mut views: Vec<ScheduledOverageEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_overage_estimate_view(item));
    }
    views
}

pub fn sum_scheduled_overage_estimate_view(view: &ScheduledOverageEstimateView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + (parsed_attempts as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_overage_estimate("code-428", "label-267");
        let view = to_scheduled_overage_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_overage_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
