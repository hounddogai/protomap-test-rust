use super::*;

/// How screens and exports show a fallback action item manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifestView {
    pub code_text: String,
    pub name_text: String,
    pub expires_at: String,
    pub maximum: String,
    pub approved: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_fallback_action_item_manifest_view(item: &FallbackActionItemManifest) -> FallbackActionItemManifestView {
    FallbackActionItemManifestView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        approved: format!("{}", item.approved),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_action_item_manifest_views(
    items: &[FallbackActionItemManifest],
) -> Vec<FallbackActionItemManifestView> {
    let mut views: Vec<FallbackActionItemManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_action_item_manifest_view(item));
    }
    views
}

pub fn sum_fallback_action_item_manifest_view(view: &FallbackActionItemManifestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_maximum + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_action_item_manifest("code-338", "name-24");
        let view = to_fallback_action_item_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_action_item_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
