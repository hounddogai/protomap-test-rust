use super::*;

/// How screens and exports show a batched delegation checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDelegationCheckpointView {
    pub code: String,
    pub label: String,
    pub visible_text: String,
    pub approved: String,
    pub batch_size: String,
    pub threshold: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_batched_delegation_checkpoint_view(item: &BatchedDelegationCheckpoint) -> BatchedDelegationCheckpointView {
    BatchedDelegationCheckpointView {
        code: item.code.clone(),
        label: item.label.clone(),
        visible_text: format!("{}", item.visible),
        approved: format!("{}", item.approved),
        batch_size: item.batch_size.to_string(),
        threshold: format!("{:.2}", item.threshold),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_delegation_checkpoint_views(
    items: &[BatchedDelegationCheckpoint],
) -> Vec<BatchedDelegationCheckpointView> {
    let mut views: Vec<BatchedDelegationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_delegation_checkpoint_view(item));
    }
    views
}

pub fn sum_batched_delegation_checkpoint_view(view: &BatchedDelegationCheckpointView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_delegation_checkpoint("code-397", "label-313");
        let view = to_batched_delegation_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_delegation_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
