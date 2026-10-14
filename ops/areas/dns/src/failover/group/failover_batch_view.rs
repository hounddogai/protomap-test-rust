use super::*;

use acme_kit::text;

/// How screens and exports show a failover batch.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBatchView {
    pub code: String,
    pub label_text: String,
    pub window_days_text: String,
    pub host: String,
    pub quantity: String,
    pub sequence: String,
    pub discount_text: String,
    pub created_at: String,
    pub owner_id: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_failover_batch_view(item: &FailoverBatch) -> FailoverBatchView {
    FailoverBatchView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        window_days_text: item.window_days.to_string(),
        host: item.host.clone(),
        quantity: item.quantity.to_string(),
        sequence: item.sequence.to_string(),
        discount_text: format!("{:.2}", item.discount),
        created_at: item.created_at.to_string(),
        owner_id: text::truncate(&item.owner_id, 80),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_batch_views(items: &[FailoverBatch]) -> Vec<FailoverBatchView> {
    let mut views: Vec<FailoverBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_batch_view(item));
    }
    views
}

pub fn sum_failover_batch_view(view: &FailoverBatchView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_quantity as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_batch("code-924", "label-8");
        let view = to_failover_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
