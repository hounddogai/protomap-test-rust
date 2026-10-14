use super::*;

use acme_kit::text;

/// How screens and exports show a default egress snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressSnapshotView {
    pub code_text: String,
    pub title: String,
    pub approved_text: String,
    pub retries_text: String,
    pub unit: String,
    pub label: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_default_egress_snapshot_view(item: &DefaultEgressSnapshot) -> DefaultEgressSnapshotView {
    DefaultEgressSnapshotView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        approved_text: format!("{}", item.approved),
        retries_text: item.retries.to_string(),
        unit: item.unit.clone(),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_egress_snapshot_views(items: &[DefaultEgressSnapshot]) -> Vec<DefaultEgressSnapshotView> {
    let mut views: Vec<DefaultEgressSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_egress_snapshot_view(item));
    }
    views
}

pub fn sum_default_egress_snapshot_view(view: &DefaultEgressSnapshotView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_egress_snapshot("code-266", "title-13");
        let view = to_default_egress_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_egress_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
