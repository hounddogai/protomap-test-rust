use super::*;

use acme_kit::text;

/// How screens and exports show a strict failover snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverSnapshotView {
    pub code_text: String,
    pub name_text: String,
    pub batch_size: String,
    pub rotation_days: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_strict_failover_snapshot_view(item: &StrictFailoverSnapshot) -> StrictFailoverSnapshotView {
    StrictFailoverSnapshotView {
        code_text: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 40),
        batch_size: item.batch_size.to_string(),
        rotation_days: item.rotation_days.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_failover_snapshot_views(items: &[StrictFailoverSnapshot]) -> Vec<StrictFailoverSnapshotView> {
    let mut views: Vec<StrictFailoverSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_failover_snapshot_view(item));
    }
    views
}

pub fn sum_strict_failover_snapshot_view(view: &StrictFailoverSnapshotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_failover_snapshot("code-867", "name-909");
        let view = to_strict_failover_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_failover_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
