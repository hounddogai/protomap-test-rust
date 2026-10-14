use super::*;

use acme_kit::text;

/// How screens and exports show a canary batch.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatchView {
    pub code: String,
    pub label_text: String,
    pub region: String,
    pub source_text: String,
    pub priority: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_canary_batch_view(item: &CanaryBatch) -> CanaryBatchView {
    CanaryBatchView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        region: item.region.clone(),
        source_text: text::truncate(&item.source, 64),
        priority: item.priority.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_batch_views(items: &[CanaryBatch]) -> Vec<CanaryBatchView> {
    let mut views: Vec<CanaryBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_batch_view(item));
    }
    views
}

pub fn sum_canary_batch_view(view: &CanaryBatchView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_priority as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_batch("code-62", "label-57");
        let view = to_canary_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
