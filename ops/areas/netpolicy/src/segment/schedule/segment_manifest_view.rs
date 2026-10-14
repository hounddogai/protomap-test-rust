use super::*;

use acme_kit::text;

/// How screens and exports show a segment manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManifestView {
    pub code_text: String,
    pub label: String,
    pub discount: String,
    pub updated_at: String,
    pub pinned_text: String,
    pub checksum_text: String,
    pub unit: String,
    pub external_ref_text: String,
    pub created_at: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_segment_manifest_view(item: &SegmentManifest) -> SegmentManifestView {
    SegmentManifestView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        updated_at: item.updated_at.to_string(),
        pinned_text: format!("{}", item.pinned),
        checksum_text: item.checksum.clone(),
        unit: item.unit.clone(),
        external_ref_text: item.external_ref.clone(),
        created_at: item.created_at.to_string(),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_manifest_views(items: &[SegmentManifest]) -> Vec<SegmentManifestView> {
    let mut views: Vec<SegmentManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_manifest_view(item));
    }
    views
}

pub fn sum_segment_manifest_view(view: &SegmentManifestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_discount + (parsed_updated_at as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_manifest("code-748", "label-779");
        let view = to_segment_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
