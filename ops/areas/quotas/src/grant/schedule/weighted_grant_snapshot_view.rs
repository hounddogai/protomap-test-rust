use super::*;

use acme_kit::text;

/// How screens and exports show a weighted grant snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub weight: String,
    pub name_text: String,
    pub cipher: String,
    pub segment: String,
    pub key_id: String,
    pub quantity: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_weighted_grant_snapshot_view(item: &WeightedGrantSnapshot) -> WeightedGrantSnapshotView {
    WeightedGrantSnapshotView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        weight: format!("{:.2}", item.weight),
        name_text: item.name.clone(),
        cipher: text::truncate(&item.cipher, 80),
        segment: item.segment.clone(),
        key_id: item.key_id.clone(),
        quantity: item.quantity.to_string(),
        scope: text::truncate(&item.scope, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_grant_snapshot_views(items: &[WeightedGrantSnapshot]) -> Vec<WeightedGrantSnapshotView> {
    let mut views: Vec<WeightedGrantSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_grant_snapshot_view(item));
    }
    views
}

pub fn sum_weighted_grant_snapshot_view(view: &WeightedGrantSnapshotView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_cost + parsed_weight + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_grant_snapshot("code-218", "label-705");
        let view = to_weighted_grant_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_grant_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
