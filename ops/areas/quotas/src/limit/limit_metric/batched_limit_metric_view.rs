use super::*;

use acme_kit::text;

/// How screens and exports show a batched limit metric.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMetricView {
    pub code_text: String,
    pub name: String,
    pub quantity: String,
    pub amount: String,
    pub principal: String,
    pub revision: String,
    pub starts_at_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_batched_limit_metric_view(item: &BatchedLimitMetric) -> BatchedLimitMetricView {
    BatchedLimitMetricView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        amount: format!("{:.2}", item.amount),
        principal: text::truncate(&item.principal, 80),
        revision: item.revision.to_string(),
        starts_at_text: item.starts_at.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_limit_metric_views(items: &[BatchedLimitMetric]) -> Vec<BatchedLimitMetricView> {
    let mut views: Vec<BatchedLimitMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_limit_metric_view(item));
    }
    views
}

pub fn sum_batched_limit_metric_view(view: &BatchedLimitMetricView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_amount + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_limit_metric("code-660", "name-923");
        let view = to_batched_limit_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_limit_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
