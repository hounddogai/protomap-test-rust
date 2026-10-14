use super::*;

use acme_kit::text;

/// How screens and exports show a default rollout batch.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutBatchView {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub checksum_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_default_rollout_batch_view(item: &DefaultRolloutBatch) -> DefaultRolloutBatchView {
    DefaultRolloutBatchView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        contact_email: item.contact_email.clone(),
        checksum_text: item.checksum.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_rollout_batch_views(items: &[DefaultRolloutBatch]) -> Vec<DefaultRolloutBatchView> {
    let mut views: Vec<DefaultRolloutBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_rollout_batch_view(item));
    }
    views
}

pub fn sum_default_rollout_batch_view(view: &DefaultRolloutBatchView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_rollout_batch("code-547", "name-681");
        let view = to_default_rollout_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_rollout_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
