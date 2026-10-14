use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpointView {
    pub code_text: String,
    pub name: String,
    pub weight_text: String,
    pub priority: String,
    pub category: String,
    pub ends_at_text: String,
    pub ratio_text: String,
    pub discount_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_allowlist_checkpoint_view(item: &AllowlistCheckpoint) -> AllowlistCheckpointView {
    AllowlistCheckpointView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        weight_text: format!("{:.2}", item.weight),
        priority: item.priority.to_string(),
        category: item.category.clone(),
        ends_at_text: item.ends_at.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        discount_text: format!("{:.2}", item.discount),
        external_ref: text::truncate(&item.external_ref, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_checkpoint_views(items: &[AllowlistCheckpoint]) -> Vec<AllowlistCheckpointView> {
    let mut views: Vec<AllowlistCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_checkpoint_view(item));
    }
    views
}

pub fn sum_allowlist_checkpoint_view(view: &AllowlistCheckpointView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_weight + (parsed_priority as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_checkpoint("code-724", "name-125");
        let view = to_allowlist_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
