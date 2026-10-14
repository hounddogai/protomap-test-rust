use super::*;

use acme_kit::text;

/// How screens and exports show a primary action item manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestView {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub active_text: String,
    pub duration_minutes: String,
    pub cluster: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_primary_action_item_manifest_view(item: &PrimaryActionItemManifest) -> PrimaryActionItemManifestView {
    PrimaryActionItemManifestView {
        code: item.code.clone(),
        label: item.label.clone(),
        principal: text::truncate(&item.principal, 40),
        active_text: format!("{}", item.active),
        duration_minutes: item.duration_minutes.to_string(),
        cluster: item.cluster.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_action_item_manifest_views(
    items: &[PrimaryActionItemManifest],
) -> Vec<PrimaryActionItemManifestView> {
    let mut views: Vec<PrimaryActionItemManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_action_item_manifest_view(item));
    }
    views
}

pub fn sum_primary_action_item_manifest_view(view: &PrimaryActionItemManifestView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_action_item_manifest("code-704", "label-675");
        let view = to_primary_action_item_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_action_item_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
