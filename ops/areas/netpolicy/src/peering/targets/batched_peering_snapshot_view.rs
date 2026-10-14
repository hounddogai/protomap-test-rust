use super::*;

use acme_kit::text;

/// How screens and exports show a batched peering snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshotView {
    pub code_text: String,
    pub name: String,
    pub reviewer: String,
    pub summary: String,
    pub audience: String,
    pub currency: String,
    pub batch_size_text: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_batched_peering_snapshot_view(item: &BatchedPeeringSnapshot) -> BatchedPeeringSnapshotView {
    BatchedPeeringSnapshotView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        reviewer: item.reviewer.clone(),
        summary: text::truncate(&item.summary, 64),
        audience: item.audience.clone(),
        currency: item.currency.clone(),
        batch_size_text: item.batch_size.to_string(),
        principal: text::truncate(&item.principal, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_peering_snapshot_views(items: &[BatchedPeeringSnapshot]) -> Vec<BatchedPeeringSnapshotView> {
    let mut views: Vec<BatchedPeeringSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_peering_snapshot_view(item));
    }
    views
}

pub fn sum_batched_peering_snapshot_view(view: &BatchedPeeringSnapshotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_peering_snapshot("code-47", "name-30");
        let view = to_batched_peering_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_peering_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
