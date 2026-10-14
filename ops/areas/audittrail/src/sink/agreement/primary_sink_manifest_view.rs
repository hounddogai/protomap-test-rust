use super::*;

use acme_kit::text;

/// How screens and exports show a primary sink manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkManifestView {
    pub code: String,
    pub label: String,
    pub ends_at: String,
    pub host_text: String,
    pub sequence: String,
    pub namespace: String,
    pub external_ref_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_primary_sink_manifest_view(item: &PrimarySinkManifest) -> PrimarySinkManifestView {
    PrimarySinkManifestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        ends_at: item.ends_at.to_string(),
        host_text: item.host.clone(),
        sequence: item.sequence.to_string(),
        namespace: item.namespace.clone(),
        external_ref_text: text::truncate(&item.external_ref, 80),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_sink_manifest_views(items: &[PrimarySinkManifest]) -> Vec<PrimarySinkManifestView> {
    let mut views: Vec<PrimarySinkManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_sink_manifest_view(item));
    }
    views
}

pub fn sum_primary_sink_manifest_view(view: &PrimarySinkManifestView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_sequence as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_sink_manifest("code-891", "label-778");
        let view = to_primary_sink_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_sink_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
