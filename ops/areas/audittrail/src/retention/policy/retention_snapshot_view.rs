use super::*;

use acme_kit::text;

/// How screens and exports show a retention snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSnapshotView {
    pub code: String,
    pub name: String,
    pub cost: String,
    pub reviewer: String,
    pub discount: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_retention_snapshot_view(item: &RetentionSnapshot) -> RetentionSnapshotView {
    RetentionSnapshotView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        reviewer: text::truncate(&item.reviewer, 64),
        discount: format!("{:.2}", item.discount),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_snapshot_views(items: &[RetentionSnapshot]) -> Vec<RetentionSnapshotView> {
    let mut views: Vec<RetentionSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_snapshot_view(item));
    }
    views
}

pub fn sum_retention_snapshot_view(view: &RetentionSnapshotView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_cost + parsed_discount + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_snapshot("code-767", "name-9");
        let view = to_retention_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
