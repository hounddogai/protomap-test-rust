use super::*;

use acme_kit::text;

/// How screens and exports show a approval manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalManifestView {
    pub code: String,
    pub name_text: String,
    pub floor_value_text: String,
    pub attempts: String,
    pub starts_at: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_approval_manifest_view(item: &ApprovalManifest) -> ApprovalManifestView {
    ApprovalManifestView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        attempts: item.attempts.to_string(),
        starts_at: item.starts_at.to_string(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_manifest_views(items: &[ApprovalManifest]) -> Vec<ApprovalManifestView> {
    let mut views: Vec<ApprovalManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_manifest_view(item));
    }
    views
}

pub fn sum_approval_manifest_view(view: &ApprovalManifestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_floor_value + (parsed_attempts as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_manifest("code-869", "name-732");
        let view = to_approval_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
