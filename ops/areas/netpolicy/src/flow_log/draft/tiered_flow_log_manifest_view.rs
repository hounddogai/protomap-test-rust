use super::*;

use acme_kit::text;

/// How screens and exports show a tiered flow log manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogManifestView {
    pub code: String,
    pub name: String,
    pub ttl_seconds_text: String,
    pub batch_size_text: String,
    pub created_at_text: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub owner_id_text: String,
    pub contact_email: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_tiered_flow_log_manifest_view(item: &TieredFlowLogManifest) -> TieredFlowLogManifestView {
    TieredFlowLogManifestView {
        code: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        batch_size_text: item.batch_size.to_string(),
        created_at_text: item.created_at.to_string(),
        external_ref: item.external_ref.clone(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        owner_id_text: item.owner_id.clone(),
        contact_email: item.contact_email.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_flow_log_manifest_views(items: &[TieredFlowLogManifest]) -> Vec<TieredFlowLogManifestView> {
    let mut views: Vec<TieredFlowLogManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_flow_log_manifest_view(item));
    }
    views
}

pub fn sum_tiered_flow_log_manifest_view(view: &TieredFlowLogManifestView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_batch_size as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_flow_log_manifest("code-344", "name-838");
        let view = to_tiered_flow_log_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_flow_log_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
