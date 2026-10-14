use super::*;

use acme_kit::text;

/// How screens and exports show a weighted zone checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpointView {
    pub code: String,
    pub label: String,
    pub revision: String,
    pub approved: String,
    pub checksum: String,
    pub audience: String,
    pub tenant_id: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_weighted_zone_checkpoint_view(item: &WeightedZoneCheckpoint) -> WeightedZoneCheckpointView {
    WeightedZoneCheckpointView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        revision: item.revision.to_string(),
        approved: format!("{}", item.approved),
        checksum: item.checksum.clone(),
        audience: item.audience.clone(),
        tenant_id: item.tenant_id.clone(),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_zone_checkpoint_views(items: &[WeightedZoneCheckpoint]) -> Vec<WeightedZoneCheckpointView> {
    let mut views: Vec<WeightedZoneCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_zone_checkpoint_view(item));
    }
    views
}

pub fn sum_weighted_zone_checkpoint_view(view: &WeightedZoneCheckpointView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_zone_checkpoint("code-102", "label-444");
        let view = to_weighted_zone_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_zone_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
