use super::*;

use acme_kit::text;

/// How screens and exports show a global freeze manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifestView {
    pub code: String,
    pub title_text: String,
    pub namespace_text: String,
    pub channel: String,
    pub checksum_text: String,
    pub margin: String,
    pub floor_value: String,
    pub segment: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_global_freeze_manifest_view(item: &GlobalFreezeManifest) -> GlobalFreezeManifestView {
    GlobalFreezeManifestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        namespace_text: item.namespace.clone(),
        channel: text::truncate(&item.channel, 80),
        checksum_text: text::truncate(&item.checksum, 40),
        margin: format!("{:.2}", item.margin),
        floor_value: format!("{:.2}", item.floor_value),
        segment: item.segment.clone(),
        principal: text::truncate(&item.principal, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_freeze_manifest_views(items: &[GlobalFreezeManifest]) -> Vec<GlobalFreezeManifestView> {
    let mut views: Vec<GlobalFreezeManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_freeze_manifest_view(item));
    }
    views
}

pub fn sum_global_freeze_manifest_view(view: &GlobalFreezeManifestView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_margin + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_freeze_manifest("code-388", "title-915");
        let view = to_global_freeze_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_freeze_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
