use super::*;

use acme_kit::text;

/// How screens and exports show a runbook bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBundleView {
    pub code_text: String,
    pub label_text: String,
    pub active_text: String,
    pub country: String,
    pub key_id_text: String,
    pub archived_text: String,
    pub region: String,
    pub timezone: String,
    pub created_at: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_runbook_bundle_view(item: &RunbookBundle) -> RunbookBundleView {
    RunbookBundleView {
        code_text: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 64),
        active_text: format!("{}", item.active),
        country: item.country.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        archived_text: format!("{}", item.archived),
        region: item.region.clone(),
        timezone: item.timezone.clone(),
        created_at: item.created_at.to_string(),
        host_text: text::truncate(&item.host, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_bundle_views(items: &[RunbookBundle]) -> Vec<RunbookBundleView> {
    let mut views: Vec<RunbookBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_bundle_view(item));
    }
    views
}

pub fn sum_runbook_bundle_view(view: &RunbookBundleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_bundle("code-596", "label-626");
        let view = to_runbook_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
