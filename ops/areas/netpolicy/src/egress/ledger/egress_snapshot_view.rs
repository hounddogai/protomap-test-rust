use super::*;

use acme_kit::text;

/// How screens and exports show a egress snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSnapshotView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub title: String,
    pub duration_minutes: String,
    pub threshold_text: String,
    pub owner_id: String,
    pub cost_text: String,
    pub maximum: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_egress_snapshot_view(item: &EgressSnapshot) -> EgressSnapshotView {
    EgressSnapshotView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        title: item.title.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        owner_id: item.owner_id.clone(),
        cost_text: format!("{:.2}", item.cost),
        maximum: format!("{:.2}", item.maximum),
        region_text: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_snapshot_views(items: &[EgressSnapshot]) -> Vec<EgressSnapshotView> {
    let mut views: Vec<EgressSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_snapshot_view(item));
    }
    views
}

pub fn sum_egress_snapshot_view(view: &EgressSnapshotView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_weight + (parsed_duration_minutes as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_snapshot("code-302", "name-762");
        let view = to_egress_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
