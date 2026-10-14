use super::*;

use acme_kit::text;

/// How screens and exports show a rollback manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifestView {
    pub code: String,
    pub name: String,
    pub summary_text: String,
    pub ends_at: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_rollback_manifest_view(item: &RollbackManifest) -> RollbackManifestView {
    RollbackManifestView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        summary_text: item.summary.clone(),
        ends_at: item.ends_at.to_string(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_manifest_views(items: &[RollbackManifest]) -> Vec<RollbackManifestView> {
    let mut views: Vec<RollbackManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_manifest_view(item));
    }
    views
}

pub fn sum_rollback_manifest_view(view: &RollbackManifestView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_manifest("code-484", "name-884");
        let view = to_rollback_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
