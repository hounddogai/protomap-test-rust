use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled suspension manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionManifestView {
    pub code: String,
    pub label_text: String,
    pub visible: String,
    pub score: String,
    pub name: String,
    pub segment: String,
    pub summary_text: String,
    pub fingerprint_text: String,
    pub archived: String,
    pub tenant_id: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_scheduled_suspension_manifest_view(item: &ScheduledSuspensionManifest) -> ScheduledSuspensionManifestView {
    ScheduledSuspensionManifestView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        visible: format!("{}", item.visible),
        score: format!("{:.2}", item.score),
        name: item.name.clone(),
        segment: text::truncate(&item.segment, 80),
        summary_text: item.summary.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 40),
        archived: format!("{}", item.archived),
        tenant_id: item.tenant_id.clone(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_suspension_manifest_views(
    items: &[ScheduledSuspensionManifest],
) -> Vec<ScheduledSuspensionManifestView> {
    let mut views: Vec<ScheduledSuspensionManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_suspension_manifest_view(item));
    }
    views
}

pub fn sum_scheduled_suspension_manifest_view(view: &ScheduledSuspensionManifestView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_score + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_suspension_manifest("code-973", "label-91");
        let view = to_scheduled_suspension_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_suspension_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
