use super::*;

/// How screens and exports show a pending grant metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetricView {
    pub code: String,
    pub title_text: String,
    pub attempts_text: String,
    pub capacity: String,
    pub country: String,
    pub timezone: String,
    pub amount: String,
    pub sequence_text: String,
    pub locale_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_pending_grant_metric_view(item: &PendingGrantMetric) -> PendingGrantMetricView {
    PendingGrantMetricView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        attempts_text: item.attempts.to_string(),
        capacity: item.capacity.to_string(),
        country: item.country.clone(),
        timezone: item.timezone.clone(),
        amount: format!("{:.2}", item.amount),
        sequence_text: item.sequence.to_string(),
        locale_text: item.locale.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_grant_metric_views(items: &[PendingGrantMetric]) -> Vec<PendingGrantMetricView> {
    let mut views: Vec<PendingGrantMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_grant_metric_view(item));
    }
    views
}

pub fn sum_pending_grant_metric_view(view: &PendingGrantMetricView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_capacity as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_grant_metric("code-594", "title-324");
        let view = to_pending_grant_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_grant_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
