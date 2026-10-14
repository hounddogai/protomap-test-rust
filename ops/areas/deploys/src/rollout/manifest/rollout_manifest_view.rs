use super::*;

use acme_kit::text;

/// How screens and exports show a rollout manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutManifestView {
    pub code: String,
    pub name_text: String,
    pub segment: String,
    pub capacity_text: String,
    pub reviewer_text: String,
    pub contact_email_text: String,
    pub active_text: String,
    pub burst: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_rollout_manifest_view(item: &RolloutManifest) -> RolloutManifestView {
    RolloutManifestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        segment: item.segment.clone(),
        capacity_text: item.capacity.to_string(),
        reviewer_text: item.reviewer.clone(),
        contact_email_text: text::truncate(&item.contact_email, 64),
        active_text: format!("{}", item.active),
        burst: item.burst.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_manifest_views(items: &[RolloutManifest]) -> Vec<RolloutManifestView> {
    let mut views: Vec<RolloutManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_manifest_view(item));
    }
    views
}

pub fn sum_rollout_manifest_view(view: &RolloutManifestView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_burst as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_manifest("code-858", "name-298");
        let view = to_rollout_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
