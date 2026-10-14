use super::*;

use acme_kit::text;

/// How screens and exports show a action item manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemManifestView {
    pub code_text: String,
    pub title: String,
    pub label: String,
    pub rate: String,
    pub channel: String,
    pub minimum: String,
    pub limit: String,
    pub floor_value: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_action_item_manifest_view(item: &ActionItemManifest) -> ActionItemManifestView {
    ActionItemManifestView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        channel: item.channel.clone(),
        minimum: format!("{:.2}", item.minimum),
        limit: item.limit.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_manifest_views(items: &[ActionItemManifest]) -> Vec<ActionItemManifestView> {
    let mut views: Vec<ActionItemManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_manifest_view(item));
    }
    views
}

pub fn sum_action_item_manifest_view(view: &ActionItemManifestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rate + parsed_minimum + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_manifest("code-376", "title-618");
        let view = to_action_item_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
