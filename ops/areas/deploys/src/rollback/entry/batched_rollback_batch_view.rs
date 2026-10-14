use super::*;

/// How screens and exports show a batched rollback batch.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub unit: String,
    pub budget: String,
    pub expires_at_text: String,
    pub rotation_days: String,
    pub title_text: String,
    pub rate: String,
    pub duration_minutes: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_batched_rollback_batch_view(item: &BatchedRollbackBatch) -> BatchedRollbackBatchView {
    BatchedRollbackBatchView {
        code: item.code.clone(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        unit: item.unit.clone(),
        budget: format!("{:.2}", item.budget),
        expires_at_text: item.expires_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        title_text: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        duration_minutes: item.duration_minutes.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_rollback_batch_views(items: &[BatchedRollbackBatch]) -> Vec<BatchedRollbackBatchView> {
    let mut views: Vec<BatchedRollbackBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_rollback_batch_view(item));
    }
    views
}

pub fn sum_batched_rollback_batch_view(view: &BatchedRollbackBatchView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_amount + parsed_budget + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_rollback_batch("code-500", "label-79");
        let view = to_batched_rollback_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_rollback_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
